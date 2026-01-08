
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DataError {
    #[error("Invalid data format")]
    InvalidFormat,
    #[error("Missing required field: {0}")]
    MissingField(String),
    #[error("Value out of range: {0}")]
    OutOfRange(String),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub timestamp: i64,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

impl DataRecord {
    pub fn new(id: u32, timestamp: i64, values: Vec<f64>) -> Self {
        Self {
            id,
            timestamp,
            values,
            metadata: HashMap::new(),
        }
    }

    pub fn validate(&self) -> Result<(), DataError> {
        if self.id == 0 {
            return Err(DataError::InvalidFormat);
        }

        if self.timestamp < 0 {
            return Err(DataError::OutOfRange("timestamp".to_string()));
        }

        if self.values.is_empty() {
            return Err(DataError::MissingField("values".to_string()));
        }

        for (i, &value) in self.values.iter().enumerate() {
            if !value.is_finite() {
                return Err(DataError::OutOfRange(format!("value at index {}", i)));
            }
        }

        Ok(())
    }

    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }
}

pub fn process_records(records: Vec<DataRecord>) -> Result<Vec<DataRecord>, DataError> {
    let mut processed = Vec::with_capacity(records.len());

    for record in records {
        record.validate()?;
        let mut processed_record = record.clone();
        
        if let Some(scale_factor) = processed_record.get_metadata("scale") {
            if let Ok(factor) = scale_factor.parse::<f64>() {
                processed_record.values = processed_record.values
                    .iter()
                    .map(|&v| v * factor)
                    .collect();
            }
        }
        
        processed.push(processed_record);
    }

    Ok(processed)
}

pub fn filter_records(records: Vec<DataRecord>, min_value: f64) -> Vec<DataRecord> {
    records
        .into_iter()
        .filter(|record| record.values.iter().any(|&v| v >= min_value))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_validation() {
        let valid_record = DataRecord::new(1, 1234567890, vec![1.0, 2.0, 3.0]);
        assert!(valid_record.validate().is_ok());

        let invalid_record = DataRecord::new(0, 1234567890, vec![1.0, 2.0]);
        assert!(invalid_record.validate().is_err());
    }

    #[test]
    fn test_metadata_operations() {
        let mut record = DataRecord::new(1, 1234567890, vec![1.0]);
        record.add_metadata("source".to_string(), "sensor_a".to_string());
        
        assert_eq!(record.get_metadata("source"), Some(&"sensor_a".to_string()));
        assert_eq!(record.get_metadata("nonexistent"), None);
    }

    #[test]
    fn test_process_records() {
        let mut record = DataRecord::new(1, 1234567890, vec![1.0, 2.0]);
        record.add_metadata("scale".to_string(), "2.0".to_string());
        
        let result = process_records(vec![record]);
        assert!(result.is_ok());
        
        let processed = result.unwrap();
        assert_eq!(processed[0].values, vec![2.0, 4.0]);
    }
}