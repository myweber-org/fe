use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub timestamp: i64,
}

#[derive(Debug)]
pub enum ProcessingError {
    InvalidValue(f64),
    InvalidTimestamp(i64),
    MissingField(String),
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessingError::InvalidValue(v) => write!(f, "Invalid value: {}", v),
            ProcessingError::InvalidTimestamp(t) => write!(f, "Invalid timestamp: {}", t),
            ProcessingError::MissingField(field) => write!(f, "Missing field: {}", field),
        }
    }
}

impl Error for ProcessingError {}

pub fn validate_record(record: &DataRecord) -> Result<(), ProcessingError> {
    if record.value.is_nan() || record.value.is_infinite() {
        return Err(ProcessingError::InvalidValue(record.value));
    }
    
    if record.timestamp < 0 {
        return Err(ProcessingError::InvalidTimestamp(record.timestamp));
    }
    
    Ok(())
}

pub fn transform_record(record: &DataRecord) -> Result<DataRecord, ProcessingError> {
    validate_record(record)?;
    
    let transformed = DataRecord {
        id: record.id,
        value: (record.value * 100.0).round() / 100.0,
        timestamp: record.timestamp,
    };
    
    Ok(transformed)
}

pub fn process_records(records: Vec<DataRecord>) -> Vec<Result<DataRecord, ProcessingError>> {
    records
        .into_iter()
        .map(|record| transform_record(&record))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_record() {
        let record = DataRecord {
            id: 1,
            value: 42.5,
            timestamp: 1625097600,
        };
        
        assert!(validate_record(&record).is_ok());
    }

    #[test]
    fn test_invalid_value() {
        let record = DataRecord {
            id: 2,
            value: f64::NAN,
            timestamp: 1625097600,
        };
        
        assert!(validate_record(&record).is_err());
    }

    #[test]
    fn test_transform_record() {
        let record = DataRecord {
            id: 3,
            value: 123.456,
            timestamp: 1625097600,
        };
        
        let transformed = transform_record(&record).unwrap();
        assert_eq!(transformed.value, 123.46);
    }
}
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DataError {
    #[error("Invalid input data")]
    InvalidInput,
    #[error("Processing timeout")]
    Timeout,
    #[error("Serialization failed")]
    SerializationFailed,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DataRecord {
    pub id: u64,
    pub timestamp: i64,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

impl DataRecord {
    pub fn new(id: u64, timestamp: i64) -> Self {
        Self {
            id,
            timestamp,
            values: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn add_value(&mut self, value: f64) {
        self.values.push(value);
    }

    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    pub fn validate(&self) -> Result<(), DataError> {
        if self.id == 0 {
            return Err(DataError::InvalidInput);
        }
        if self.timestamp < 0 {
            return Err(DataError::InvalidInput);
        }
        if self.values.is_empty() {
            return Err(DataError::InvalidInput);
        }
        Ok(())
    }
}

pub fn process_records(records: Vec<DataRecord>) -> Result<Vec<DataRecord>, DataError> {
    let mut processed = Vec::with_capacity(records.len());
    
    for mut record in records {
        record.validate()?;
        
        let sum: f64 = record.values.iter().sum();
        let avg = sum / record.values.len() as f64;
        
        record.add_metadata("processed".to_string(), "true".to_string());
        record.add_metadata("average".to_string(), avg.to_string());
        
        processed.push(record);
    }
    
    Ok(processed)
}

pub fn filter_records(records: Vec<DataRecord>, min_value: f64) -> Vec<DataRecord> {
    records
        .into_iter()
        .filter(|record| {
            record.values.iter().any(|&v| v >= min_value)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_validation() {
        let mut record = DataRecord::new(1, 1000);
        record.add_value(42.0);
        
        assert!(record.validate().is_ok());
    }

    #[test]
    fn test_invalid_record() {
        let record = DataRecord::new(0, 1000);
        assert!(record.validate().is_err());
    }

    #[test]
    fn test_process_records() {
        let mut record = DataRecord::new(1, 1000);
        record.add_value(10.0);
        record.add_value(20.0);
        
        let result = process_records(vec![record.clone()]);
        assert!(result.is_ok());
        
        let processed = result.unwrap();
        assert_eq!(processed[0].metadata.get("processed").unwrap(), "true");
    }
}