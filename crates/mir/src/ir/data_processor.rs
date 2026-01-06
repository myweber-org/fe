use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub category: String,
    pub timestamp: i64,
}

#[derive(Debug)]
pub enum ProcessingError {
    InvalidValue,
    InvalidCategory,
    TimestampError,
    SerializationError(String),
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessingError::InvalidValue => write!(f, "Invalid numeric value"),
            ProcessingError::InvalidCategory => write!(f, "Invalid category string"),
            ProcessingError::TimestampError => write!(f, "Invalid timestamp"),
            ProcessingError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
        }
    }
}

impl Error for ProcessingError {}

pub struct DataProcessor {
    validation_threshold: f64,
    allowed_categories: Vec<String>,
}

impl DataProcessor {
    pub fn new(threshold: f64, categories: Vec<String>) -> Self {
        DataProcessor {
            validation_threshold: threshold,
            allowed_categories: categories,
        }
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if record.value.abs() > self.validation_threshold {
            return Err(ProcessingError::InvalidValue);
        }

        if !self.allowed_categories.contains(&record.category) {
            return Err(ProcessingError::InvalidCategory);
        }

        if record.timestamp < 0 {
            return Err(ProcessingError::TimestampError);
        }

        Ok(())
    }

    pub fn transform_record(&self, record: &DataRecord) -> DataRecord {
        let normalized_value = if record.value < 0.0 {
            record.value.abs()
        } else {
            record.value
        };

        let processed_category = record.category.to_uppercase();

        DataRecord {
            id: record.id,
            value: normalized_value,
            category: processed_category,
            timestamp: record.timestamp,
        }
    }

    pub fn process_batch(&self, records: Vec<DataRecord>) -> Vec<Result<DataRecord, ProcessingError>> {
        records
            .into_iter()
            .map(|record| {
                self.validate_record(&record)
                    .map(|_| self.transform_record(&record))
            })
            .collect()
    }

    pub fn serialize_to_json(&self, record: &DataRecord) -> Result<String, ProcessingError> {
        serde_json::to_string(record)
            .map_err(|e| ProcessingError::SerializationError(e.to_string()))
    }

    pub fn deserialize_from_json(json_str: &str) -> Result<DataRecord, ProcessingError> {
        serde_json::from_str(json_str)
            .map_err(|e| ProcessingError::SerializationError(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_success() {
        let processor = DataProcessor::new(
            1000.0,
            vec!["A".to_string(), "B".to_string(), "C".to_string()]
        );
        
        let record = DataRecord {
            id: 1,
            value: 500.0,
            category: "A".to_string(),
            timestamp: 1234567890,
        };

        assert!(processor.validate_record(&record).is_ok());
    }

    #[test]
    fn test_validation_failure() {
        let processor = DataProcessor::new(
            100.0,
            vec!["VALID".to_string()]
        );
        
        let record = DataRecord {
            id: 1,
            value: 150.0,
            category: "INVALID".to_string(),
            timestamp: 1234567890,
        };

        assert!(processor.validate_record(&record).is_err());
    }

    #[test]
    fn test_transform_record() {
        let processor = DataProcessor::new(1000.0, vec![]);
        
        let record = DataRecord {
            id: 1,
            value: -50.5,
            category: "test".to_string(),
            timestamp: 1234567890,
        };

        let transformed = processor.transform_record(&record);
        
        assert_eq!(transformed.value, 50.5);
        assert_eq!(transformed.category, "TEST");
        assert_eq!(transformed.id, 1);
    }

    #[test]
    fn test_serialization_roundtrip() {
        let processor = DataProcessor::new(1000.0, vec![]);
        
        let original = DataRecord {
            id: 42,
            value: 3.14,
            category: "PI".to_string(),
            timestamp: 1609459200,
        };

        let json = processor.serialize_to_json(&original).unwrap();
        let deserialized = DataProcessor::deserialize_from_json(&json).unwrap();

        assert_eq!(original.id, deserialized.id);
        assert_eq!(original.value, deserialized.value);
        assert_eq!(original.category, deserialized.category);
        assert_eq!(original.timestamp, deserialized.timestamp);
    }
}