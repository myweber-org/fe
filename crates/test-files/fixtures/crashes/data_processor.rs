
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

        if let Some(cached) = self.cache.get(key) {
            return Ok(cached.clone());
        }

        let processed: Vec<f64> = values
            .iter()
            .filter(|&&x| x.is_finite())
            .map(|&x| x * 2.0)
            .collect();

        if processed.len() < values.len() / 2 {
            return Err("Too many invalid values in input data".to_string());
        }

        self.cache.insert(key.to_string(), processed.clone());
        Ok(processed)
    }

    pub fn calculate_statistics(&self, data: &[f64]) -> (f64, f64, f64) {
        if data.is_empty() {
            return (0.0, 0.0, 0.0);
        }

        let sum: f64 = data.iter().sum();
        let mean = sum / data.len() as f64;

        let variance: f64 = data
            .iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / data.len() as f64;

        let std_dev = variance.sqrt();

        (mean, variance, std_dev)
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    pub fn cache_size(&self) -> usize {
        self.cache.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_numeric_data() {
        let mut processor = DataProcessor::new();
        let data = vec![1.0, 2.0, 3.0];
        let result = processor.process_numeric_data("test", &data).unwrap();
        assert_eq!(result, vec![2.0, 4.0, 6.0]);
    }

    #[test]
    fn test_calculate_statistics() {
        let processor = DataProcessor::new();
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let (mean, variance, std_dev) = processor.calculate_statistics(&data);
        assert_eq!(mean, 3.0);
        assert_eq!(variance, 2.0);
        assert_eq!(std_dev, 2.0_f64.sqrt());
    }
}
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

    pub fn process_numeric_data(&mut self, key: &str, data: &[f64]) -> Result<Vec<f64>, String> {
        if data.is_empty() {
            return Err("Empty data provided".to_string());
        }

        if let Some(cached) = self.cache.get(key) {
            return Ok(cached.clone());
        }

        let mean = data.iter().sum::<f64>() / data.len() as f64;
        let variance: f64 = data.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / data.len() as f64;
        let std_dev = variance.sqrt();

        let normalized: Vec<f64> = data
            .iter()
            .map(|&x| if std_dev > 0.0 { (x - mean) / std_dev } else { 0.0 })
            .collect();

        self.cache.insert(key.to_string(), normalized.clone());
        Ok(normalized)
    }

    pub fn filter_outliers(&self, data: &[f64], threshold: f64) -> Vec<f64> {
        if data.len() < 3 {
            return data.to_vec();
        }

        let sorted: Vec<f64> = {
            let mut temp = data.to_vec();
            temp.sort_by(|a, b| a.partial_cmp(b).unwrap());
            temp
        };

        let q1_index = (sorted.len() as f64 * 0.25).floor() as usize;
        let q3_index = (sorted.len() as f64 * 0.75).floor() as usize;
        let q1 = sorted[q1_index];
        let q3 = sorted[q3_index];
        let iqr = q3 - q1;

        let lower_bound = q1 - threshold * iqr;
        let upper_bound = q3 + threshold * iqr;

        data.iter()
            .filter(|&&x| x >= lower_bound && x <= upper_bound)
            .copied()
            .collect()
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    pub fn cache_size(&self) -> usize {
        self.cache.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_numeric_data() {
        let mut processor = DataProcessor::new();
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        
        let result = processor.process_numeric_data("test", &data);
        assert!(result.is_ok());
        
        let normalized = result.unwrap();
        assert_eq!(normalized.len(), 5);
    }

    #[test]
    fn test_filter_outliers() {
        let processor = DataProcessor::new();
        let data = vec![1.0, 2.0, 3.0, 100.0, 4.0, 5.0];
        
        let filtered = processor.filter_outliers(&data, 1.5);
        assert!(filtered.len() < data.len());
        assert!(!filtered.contains(&100.0));
    }

    #[test]
    fn test_empty_data() {
        let mut processor = DataProcessor::new();
        let result = processor.process_numeric_data("empty", &[]);
        assert!(result.is_err());
    }
}
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRecord {
    pub id: u64,
    pub timestamp: i64,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Error)]
pub enum ProcessingError {
    #[error("Invalid data format")]
    InvalidFormat,
    #[error("Data validation failed: {0}")]
    ValidationFailed(String),
    #[error("Transformation error: {0}")]
    TransformationError(String),
}

pub struct DataProcessor {
    validation_threshold: f64,
    transformation_factor: f64,
}

impl DataProcessor {
    pub fn new(validation_threshold: f64, transformation_factor: f64) -> Self {
        Self {
            validation_threshold,
            transformation_factor,
        }
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if record.values.is_empty() {
            return Err(ProcessingError::ValidationFailed(
                "Empty values array".to_string(),
            ));
        }

        for value in &record.values {
            if value.is_nan() || value.is_infinite() {
                return Err(ProcessingError::ValidationFailed(
                    "Invalid numeric value".to_string(),
                ));
            }

            if value.abs() > self.validation_threshold {
                return Err(ProcessingError::ValidationFailed(format!(
                    "Value {} exceeds threshold {}",
                    value, self.validation_threshold
                )));
            }
        }

        Ok(())
    }

    pub fn transform_values(&self, record: &mut DataRecord) -> Result<(), ProcessingError> {
        for value in &mut record.values {
            *value *= self.transformation_factor;

            if value.is_nan() || value.is_infinite() {
                return Err(ProcessingError::TransformationError(
                    "Transformation produced invalid value".to_string(),
                ));
            }
        }

        record.metadata.insert(
            "processed".to_string(),
            chrono::Utc::now().to_rfc3339(),
        );

        Ok(())
    }

    pub fn process_batch(
        &self,
        records: Vec<DataRecord>,
    ) -> Result<Vec<DataRecord>, ProcessingError> {
        let mut processed_records = Vec::with_capacity(records.len());

        for mut record in records {
            self.validate_record(&record)?;
            self.transform_values(&mut record)?;
            processed_records.push(record);
        }

        Ok(processed_records)
    }

    pub fn calculate_statistics(&self, records: &[DataRecord]) -> HashMap<String, f64> {
        let mut stats = HashMap::new();

        let total_values: usize = records.iter().map(|r| r.values.len()).sum();
        let sum: f64 = records
            .iter()
            .flat_map(|r| r.values.iter())
            .copied()
            .sum();

        stats.insert("total_records".to_string(), records.len() as f64);
        stats.insert("total_values".to_string(), total_values as f64);
        stats.insert("sum".to_string(), sum);

        if total_values > 0 {
            let mean = sum / total_values as f64;
            stats.insert("mean".to_string(), mean);

            let variance: f64 = records
                .iter()
                .flat_map(|r| r.values.iter())
                .map(|v| (v - mean).powi(2))
                .sum::<f64>()
                / total_values as f64;

            stats.insert("variance".to_string(), variance);
            stats.insert("std_dev".to_string(), variance.sqrt());
        }

        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_success() {
        let processor = DataProcessor::new(1000.0, 2.0);
        let record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: vec![10.0, 20.0, 30.0],
            metadata: HashMap::new(),
        };

        assert!(processor.validate_record(&record).is_ok());
    }

    #[test]
    fn test_validation_threshold_exceeded() {
        let processor = DataProcessor::new(100.0, 2.0);
        let record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: vec![10.0, 200.0, 30.0],
            metadata: HashMap::new(),
        };

        assert!(processor.validate_record(&record).is_err());
    }

    #[test]
    fn test_transformation() {
        let processor = DataProcessor::new(1000.0, 2.0);
        let mut record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: vec![10.0, 20.0, 30.0],
            metadata: HashMap::new(),
        };

        assert!(processor.transform_values(&mut record).is_ok());
        assert_eq!(record.values, vec![20.0, 40.0, 60.0]);
        assert!(record.metadata.contains_key("processed"));
    }

    #[test]
    fn test_batch_processing() {
        let processor = DataProcessor::new(1000.0, 2.0);
        let records = vec![
            DataRecord {
                id: 1,
                timestamp: 1234567890,
                values: vec![10.0, 20.0],
                metadata: HashMap::new(),
            },
            DataRecord {
                id: 2,
                timestamp: 1234567891,
                values: vec![30.0, 40.0],
                metadata: HashMap::new(),
            },
        ];

        let result = processor.process_batch(records);
        assert!(result.is_ok());
        let processed = result.unwrap();
        assert_eq!(processed.len(), 2);
        assert_eq!(processed[0].values, vec![20.0, 40.0]);
        assert_eq!(processed[1].values, vec![60.0, 80.0]);
    }
}