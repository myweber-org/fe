
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Serialize, Deserialize)]
pub struct DataRecord {
    pub id: u64,
    pub timestamp: i64,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

#[derive(Error, Debug)]
pub enum ProcessingError {
    #[error("Invalid data format")]
    InvalidFormat,
    #[error("Data validation failed: {0}")]
    ValidationFailed(String),
    #[error("Transformation error: {0}")]
    TransformationError(String),
}

pub fn validate_record(record: &DataRecord) -> Result<(), ProcessingError> {
    if record.id == 0 {
        return Err(ProcessingError::ValidationFailed("ID cannot be zero".to_string()));
    }
    
    if record.timestamp < 0 {
        return Err(ProcessingError::ValidationFailed("Timestamp cannot be negative".to_string()));
    }
    
    if record.values.is_empty() {
        return Err(ProcessingError::ValidationFailed("Values cannot be empty".to_string()));
    }
    
    Ok(())
}

pub fn normalize_values(record: &mut DataRecord) -> Result<(), ProcessingError> {
    if record.values.iter().any(|&v| v.is_nan() || v.is_infinite()) {
        return Err(ProcessingError::TransformationError("Invalid numeric values".to_string()));
    }
    
    let min_val = record.values
        .iter()
        .copied()
        .fold(f64::INFINITY, f64::min);
    
    let max_val = record.values
        .iter()
        .copied()
        .fold(f64::NEG_INFINITY, f64::max);
    
    if (max_val - min_val).abs() < f64::EPSILON {
        return Err(ProcessingError::TransformationError("Cannot normalize constant values".to_string()));
    }
    
    for value in &mut record.values {
        *value = (*value - min_val) / (max_val - min_val);
    }
    
    Ok(())
}

pub fn process_record(mut record: DataRecord) -> Result<DataRecord, ProcessingError> {
    validate_record(&record)?;
    normalize_values(&mut record)?;
    
    record.metadata.insert("processed".to_string(), "true".to_string());
    record.metadata.insert("normalized".to_string(), "true".to_string());
    
    Ok(record)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_valid_record_processing() {
        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), "test".to_string());
        
        let record = DataRecord {
            id: 1,
            timestamp: 1625097600,
            values: vec![1.0, 2.0, 3.0, 4.0],
            metadata,
        };
        
        let result = process_record(record);
        assert!(result.is_ok());
        
        let processed = result.unwrap();
        assert_eq!(processed.metadata.get("processed"), Some(&"true".to_string()));
        assert_eq!(processed.values[0], 0.0);
        assert_eq!(processed.values[3], 1.0);
    }
    
    #[test]
    fn test_invalid_id() {
        let record = DataRecord {
            id: 0,
            timestamp: 1625097600,
            values: vec![1.0, 2.0],
            metadata: HashMap::new(),
        };
        
        let result = process_record(record);
        assert!(matches!(result, Err(ProcessingError::ValidationFailed(_))));
    }
}