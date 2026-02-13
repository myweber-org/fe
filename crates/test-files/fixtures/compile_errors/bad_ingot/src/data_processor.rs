use csv::Reader;
use serde::Deserialize;
use std::error::Error;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

pub struct DataProcessor {
    records: Vec<Record>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let mut rdr = Reader::from_path(path)?;
        for result in rdr.deserialize() {
            let record: Record = result?;
            self.records.push(record);
        }
        Ok(())
    }

    pub fn validate_records(&self) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|r| r.value >= 0.0 && !r.name.is_empty())
            .collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        let valid_records = self.validate_records();
        if valid_records.is_empty() {
            return None;
        }
        let sum: f64 = valid_records.iter().map(|r| r.value).sum();
        Some(sum / valid_records.len() as f64)
    }

    pub fn group_by_category(&self) -> std::collections::HashMap<String, Vec<&Record>> {
        let mut groups = std::collections::HashMap::new();
        for record in &self.records {
            groups
                .entry(record.category.clone())
                .or_insert_with(Vec::new)
                .push(record);
        }
        groups
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processing() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            "id,name,value,category\n1,ItemA,10.5,Alpha\n2,ItemB,-3.2,Beta\n3,,15.7,Alpha"
        )
        .unwrap();

        let mut processor = DataProcessor::new();
        processor.load_from_csv(file.path()).unwrap();

        assert_eq!(processor.records.len(), 3);
        assert_eq!(processor.validate_records().len(), 2);
        assert_eq!(processor.calculate_average(), Some(13.1));

        let groups = processor.group_by_category();
        assert_eq!(groups.get("Alpha").unwrap().len(), 2);
        assert_eq!(groups.get("Beta").unwrap().len(), 1);
    }
}
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DataError {
    #[error("Invalid input data")]
    InvalidInput,
    #[error("Transformation failed")]
    TransformationFailed,
    #[error("Validation error: {0}")]
    ValidationError(String),
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
            return Err(DataError::ValidationError("Value must be finite".to_string()));
        }
        if self.timestamp < 0 {
            return Err(DataError::ValidationError("Timestamp cannot be negative".to_string()));
        }
        Ok(())
    }
}

pub fn process_records(records: Vec<DataRecord>) -> Result<Vec<DataRecord>, DataError> {
    let mut processed = Vec::with_capacity(records.len());
    
    for record in records {
        record.validate()?;
        
        let processed_record = DataRecord {
            id: record.id,
            value: transform_value(record.value)?,
            timestamp: record.timestamp,
        };
        
        processed.push(processed_record);
    }
    
    Ok(processed)
}

fn transform_value(value: f64) -> Result<f64, DataError> {
    if value.abs() > 1_000_000.0 {
        return Err(DataError::TransformationFailed);
    }
    
    let transformed = value.log10();
    if transformed.is_nan() || transformed.is_infinite() {
        Err(DataError::TransformationFailed)
    } else {
        Ok(transformed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_record() {
        let record = DataRecord {
            id: 1,
            value: 100.0,
            timestamp: 1625097600,
        };
        assert!(record.validate().is_ok());
    }

    #[test]
    fn test_invalid_record() {
        let record = DataRecord {
            id: 0,
            value: 100.0,
            timestamp: 1625097600,
        };
        assert!(record.validate().is_err());
    }

    #[test]
    fn test_process_records() {
        let records = vec![
            DataRecord {
                id: 1,
                value: 100.0,
                timestamp: 1625097600,
            },
            DataRecord {
                id: 2,
                value: 1000.0,
                timestamp: 1625097601,
            },
        ];
        
        let result = process_records(records);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 2);
    }
}use std::collections::HashMap;

pub struct DataProcessor {
    cache: HashMap<String, Vec<f64>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            cache: HashMap::new(),
        }
    }

    pub fn process_dataset(&mut self, key: &str, data: &[f64]) -> Result<Vec<f64>, String> {
        if data.is_empty() {
            return Err("Empty dataset provided".to_string());
        }

        if let Some(cached) = self.cache.get(key) {
            return Ok(cached.clone());
        }

        let validated = self.validate_data(data)?;
        let normalized = self.normalize_data(&validated);
        let transformed = self.apply_transformations(&normalized);

        self.cache.insert(key.to_string(), transformed.clone());
        Ok(transformed)
    }

    fn validate_data(&self, data: &[f64]) -> Result<Vec<f64>, String> {
        if data.iter().any(|&x| x.is_nan() || x.is_infinite()) {
            return Err("Invalid numeric values detected".to_string());
        }
        Ok(data.to_vec())
    }

    fn normalize_data(&self, data: &[f64]) -> Vec<f64> {
        let mean = data.iter().sum::<f64>() / data.len() as f64;
        let variance = data.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / data.len() as f64;
        let std_dev = variance.sqrt();

        if std_dev.abs() < 1e-10 {
            return vec![0.0; data.len()];
        }

        data.iter()
            .map(|&x| (x - mean) / std_dev)
            .collect()
    }

    fn apply_transformations(&self, data: &[f64]) -> Vec<f64> {
        data.iter()
            .map(|&x| x.powi(2).ln_1p().tanh())
            .collect()
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    pub fn cache_stats(&self) -> (usize, usize) {
        let total_items: usize = self.cache.values().map(|v| v.len()).sum();
        (self.cache.len(), total_items)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_processor_validation() {
        let mut processor = DataProcessor::new();
        let invalid_data = [1.0, f64::NAN, 3.0];
        let result = processor.process_dataset("test", &invalid_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_normalization() {
        let processor = DataProcessor::new();
        let data = [1.0, 2.0, 3.0, 4.0, 5.0];
        let normalized = processor.normalize_data(&data);
        
        let sum: f64 = normalized.iter().sum();
        let sum_sq: f64 = normalized.iter().map(|x| x.powi(2)).sum();
        
        assert!(sum.abs() < 1e-10);
        assert!((sum_sq - (data.len() as f64 - 1.0)).abs() < 1e-10);
    }
}