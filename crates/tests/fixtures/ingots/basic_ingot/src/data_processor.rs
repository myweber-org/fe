
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
    SerializationError(String),
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessingError::InvalidValue => write!(f, "Invalid data value"),
            ProcessingError::InvalidTimestamp => write!(f, "Invalid timestamp"),
            ProcessingError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
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

        Ok(())
    }

    pub fn process_record(&self, record: &DataRecord) -> Result<DataRecord, ProcessingError> {
        self.validate_record(record)?;

        let processed_value = if record.value > self.threshold {
            record.value * 0.9
        } else {
            record.value * 1.1
        };

        Ok(DataRecord {
            id: record.id,
            value: processed_value,
            timestamp: record.timestamp,
        })
    }

    pub fn serialize_record(&self, record: &DataRecord) -> Result<String, ProcessingError> {
        serde_json::to_string(record)
            .map_err(|e| ProcessingError::SerializationError(e.to_string()))
    }

    pub fn deserialize_record(&self, data: &str) -> Result<DataRecord, ProcessingError> {
        serde_json::from_str(data)
            .map_err(|e| ProcessingError::SerializationError(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_valid_record() {
        let processor = DataProcessor::new(100.0);
        let record = DataRecord {
            id: 1,
            value: 50.0,
            timestamp: 1234567890,
        };
        assert!(processor.validate_record(&record).is_ok());
    }

    #[test]
    fn test_validation_invalid_value() {
        let processor = DataProcessor::new(100.0);
        let record = DataRecord {
            id: 1,
            value: f64::NAN,
            timestamp: 1234567890,
        };
        assert!(processor.validate_record(&record).is_err());
    }

    #[test]
    fn test_processing_above_threshold() {
        let processor = DataProcessor::new(100.0);
        let record = DataRecord {
            id: 1,
            value: 150.0,
            timestamp: 1234567890,
        };
        let processed = processor.process_record(&record).unwrap();
        assert_eq!(processed.value, 135.0);
    }

    #[test]
    fn test_serialization_roundtrip() {
        let processor = DataProcessor::new(100.0);
        let original = DataRecord {
            id: 42,
            value: 75.5,
            timestamp: 987654321,
        };
        
        let serialized = processor.serialize_record(&original).unwrap();
        let deserialized = processor.deserialize_record(&serialized).unwrap();
        
        assert_eq!(original.id, deserialized.id);
        assert_eq!(original.value, deserialized.value);
        assert_eq!(original.timestamp, deserialized.timestamp);
    }
}