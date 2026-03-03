
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub timestamp: i64,
}

#[derive(Debug)]
pub enum ProcessingError {
    InvalidValue,
    InvalidTimestamp,
    ValidationFailed(String),
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessingError::InvalidValue => write!(f, "Invalid data value"),
            ProcessingError::InvalidTimestamp => write!(f, "Invalid timestamp"),
            ProcessingError::ValidationFailed(msg) => write!(f, "Validation failed: {}", msg),
        }
    }
}

impl Error for ProcessingError {}

pub struct DataProcessor {
    threshold: f64,
}

impl DataProcessor {
    pub fn new(threshold: f64) -> Self {
        DataProcessor { threshold }
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if record.value.is_nan() || record.value.is_infinite() {
            return Err(ProcessingError::InvalidValue);
        }

        if record.timestamp < 0 {
            return Err(ProcessingError::InvalidTimestamp);
        }

        if record.value > self.threshold {
            return Err(ProcessingError::ValidationFailed(
                format!("Value {} exceeds threshold {}", record.value, self.threshold)
            ));
        }

        Ok(())
    }

    pub fn transform_records(&self, records: Vec<DataRecord>) -> Vec<DataRecord> {
        records
            .into_iter()
            .filter_map(|mut record| {
                if self.validate_record(&record).is_ok() {
                    record.value = record.value * 2.0;
                    Some(record)
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn process_batch(&self, records: Vec<DataRecord>) -> Result<Vec<DataRecord>, ProcessingError> {
        let mut valid_records = Vec::new();
        
        for record in records {
            self.validate_record(&record)?;
            valid_records.push(record);
        }

        let transformed = self.transform_records(valid_records);
        Ok(transformed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_record() {
        let processor = DataProcessor::new(100.0);
        let record = DataRecord {
            id: 1,
            value: 50.0,
            timestamp: 1625097600,
        };
        
        assert!(processor.validate_record(&record).is_ok());
    }

    #[test]
    fn test_invalid_value() {
        let processor = DataProcessor::new(100.0);
        let record = DataRecord {
            id: 1,
            value: f64::NAN,
            timestamp: 1625097600,
        };
        
        assert!(processor.validate_record(&record).is_err());
    }

    #[test]
    fn test_transform_records() {
        let processor = DataProcessor::new(100.0);
        let records = vec![
            DataRecord { id: 1, value: 10.0, timestamp: 1625097600 },
            DataRecord { id: 2, value: 20.0, timestamp: 1625097601 },
        ];
        
        let transformed = processor.transform_records(records);
        assert_eq!(transformed.len(), 2);
        assert_eq!(transformed[0].value, 20.0);
        assert_eq!(transformed[1].value, 40.0);
    }
}