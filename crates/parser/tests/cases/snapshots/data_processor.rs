
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
    pub values: HashMap<String, f64>,
    pub tags: Vec<String>,
}

impl DataRecord {
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
        
        Ok(())
    }
    
    pub fn normalize_values(&mut self) {
        let sum: f64 = self.values.values().sum();
        if sum != 0.0 {
            for value in self.values.values_mut() {
                *value /= sum;
            }
        }
    }
    
    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.iter().any(|t| t == tag)
    }
}

pub fn process_records(records: &mut [DataRecord]) -> Result<Vec<DataRecord>, DataError> {
    let mut processed = Vec::with_capacity(records.len());
    
    for record in records.iter_mut() {
        record.validate()?;
        record.normalize_values();
        processed.push(record.clone());
    }
    
    Ok(processed)
}

pub fn filter_by_tag(records: &[DataRecord], tag: &str) -> Vec<DataRecord> {
    records
        .iter()
        .filter(|r| r.has_tag(tag))
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_record_validation() {
        let mut record = DataRecord {
            id: 0,
            timestamp: 1234567890,
            values: HashMap::from([("temp".to_string(), 25.5)]),
            tags: vec!["sensor".to_string()],
        };
        
        assert!(record.validate().is_err());
        
        record.id = 1;
        assert!(record.validate().is_ok());
    }
    
    #[test]
    fn test_normalize_values() {
        let mut record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: HashMap::from([
                ("a".to_string(), 10.0),
                ("b".to_string(), 20.0),
                ("c".to_string(), 30.0),
            ]),
            tags: vec![],
        };
        
        record.normalize_values();
        
        let total: f64 = record.values.values().sum();
        assert!((total - 1.0).abs() < 0.0001);
    }
}