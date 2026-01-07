
use std::collections::HashMap;

pub struct DataProcessor {
    cache: HashMap<String, Vec<f64>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            cache: HashMap::new(),
        }
    }

    pub fn process_numeric_data(&mut self, key: &str, values: &[f64]) -> Result<Vec<f64>, String> {
        if values.is_empty() {
            return Err("Empty data array provided".to_string());
        }

        if values.iter().any(|&x| x.is_nan() || x.is_infinite()) {
            return Err("Invalid numeric values detected".to_string());
        }

        let processed: Vec<f64> = values
            .iter()
            .map(|&x| x * 2.0)
            .collect();

        self.cache.insert(key.to_string(), processed.clone());

        Ok(processed)
    }

    pub fn get_cached_result(&self, key: &str) -> Option<&Vec<f64>> {
        self.cache.get(key)
    }

    pub fn calculate_statistics(&self, key: &str) -> Option<(f64, f64, f64)> {
        self.cache.get(key).map(|values| {
            let sum: f64 = values.iter().sum();
            let mean = sum / values.len() as f64;
            let variance: f64 = values.iter()
                .map(|&x| (x - mean).powi(2))
                .sum::<f64>() / values.len() as f64;
            let std_dev = variance.sqrt();

            (mean, variance, std_dev)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_processing() {
        let mut processor = DataProcessor::new();
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        
        let result = processor.process_numeric_data("test", &data);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec![2.0, 4.0, 6.0, 8.0, 10.0]);
    }

    #[test]
    fn test_invalid_data() {
        let mut processor = DataProcessor::new();
        let data = vec![1.0, f64::NAN, 3.0];
        
        let result = processor.process_numeric_data("invalid", &data);
        assert!(result.is_err());
    }

    #[test]
    fn test_statistics_calculation() {
        let mut processor = DataProcessor::new();
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        
        processor.process_numeric_data("stats", &data).unwrap();
        let stats = processor.calculate_statistics("stats").unwrap();
        
        assert!((stats.0 - 6.0).abs() < 0.001);
        assert!((stats.1 - 40.0).abs() < 0.001);
        assert!((stats.2 - 6.3245).abs() < 0.001);
    }
}
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DataError {
    #[error("Invalid input data: {0}")]
    ValidationError(String),
    #[error("Transformation failed: {0}")]
    TransformationError(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRecord {
    pub id: u64,
    pub value: f64,
    pub timestamp: i64,
}

impl DataRecord {
    pub fn validate(&self) -> Result<(), DataError> {
        if self.id == 0 {
            return Err(DataError::ValidationError("ID cannot be zero".to_string()));
        }
        if self.value.is_nan() || self.value.is_infinite() {
            return Err(DataError::ValidationError("Value must be a finite number".to_string()));
        }
        if self.timestamp < 0 {
            return Err(DataError::ValidationError("Timestamp cannot be negative".to_string()));
        }
        Ok(())
    }
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    pub fn add_record(&mut self, record: DataRecord) -> Result<(), DataError> {
        record.validate()?;
        self.records.push(record);
        Ok(())
    }

    pub fn process_records(&self) -> Result<Vec<f64>, DataError> {
        if self.records.is_empty() {
            return Err(DataError::TransformationError("No records to process".to_string()));
        }

        let mut results = Vec::with_capacity(self.records.len());
        for record in &self.records {
            let processed_value = record.value * 2.0 - 1.0;
            if processed_value.is_nan() || processed_value.is_infinite() {
                return Err(DataError::TransformationError(
                    format!("Transformation produced invalid value for record {}", record.id)
                ));
            }
            results.push(processed_value);
        }
        Ok(results)
    }

    pub fn calculate_statistics(&self) -> Result<(f64, f64, f64), DataError> {
        if self.records.is_empty() {
            return Err(DataError::TransformationError("Cannot calculate statistics on empty dataset".to_string()));
        }

        let values: Vec<f64> = self.records.iter().map(|r| r.value).collect();
        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let variance = values.iter()
            .map(|v| (v - mean).powi(2))
            .sum::<f64>() / values.len() as f64;
        let std_dev = variance.sqrt();

        Ok((mean, variance, std_dev))
    }

    pub fn save_to_file(&self, path: &str) -> Result<(), DataError> {
        let json_data = serde_json::to_string_pretty(&self.records)
            .map_err(|e| DataError::IoError(std::io::Error::new(std::io::ErrorKind::InvalidData, e)))?;
        
        std::fs::write(path, json_data)?;
        Ok(())
    }

    pub fn load_from_file(path: &str) -> Result<Self, DataError> {
        let content = std::fs::read_to_string(path)?;
        let records: Vec<DataRecord> = serde_json::from_str(&content)
            .map_err(|e| DataError::IoError(std::io::Error::new(std::io::ErrorKind::InvalidData, e)))?;
        
        let mut processor = DataProcessor::new();
        for record in records {
            processor.add_record(record)?;
        }
        Ok(processor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_record() {
        let record = DataRecord {
            id: 1,
            value: 42.5,
            timestamp: 1234567890,
        };
        assert!(record.validate().is_ok());
    }

    #[test]
    fn test_invalid_record_zero_id() {
        let record = DataRecord {
            id: 0,
            value: 42.5,
            timestamp: 1234567890,
        };
        assert!(record.validate().is_err());
    }

    #[test]
    fn test_data_processor_operations() {
        let mut processor = DataProcessor::new();
        
        let record = DataRecord {
            id: 1,
            value: 10.0,
            timestamp: 1000,
        };
        
        assert!(processor.add_record(record).is_ok());
        assert_eq!(processor.records.len(), 1);
        
        let stats = processor.calculate_statistics().unwrap();
        assert_eq!(stats.0, 10.0);
        assert_eq!(stats.1, 0.0);
        assert_eq!(stats.2, 0.0);
    }
}