
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub timestamp: i64,
    pub category: String,
}

#[derive(Debug)]
pub enum ProcessingError {
    InvalidValue,
    InvalidTimestamp,
    CategoryNotFound,
    SerializationError(String),
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessingError::InvalidValue => write!(f, "Value is outside valid range"),
            ProcessingError::InvalidTimestamp => write!(f, "Timestamp is invalid"),
            ProcessingError::CategoryNotFound => write!(f, "Category does not exist"),
            ProcessingError::SerializationError(msg) => write!(f, "Serialization failed: {}", msg),
        }
    }
}

impl Error for ProcessingError {}

pub struct DataProcessor {
    valid_categories: Vec<String>,
    min_value: f64,
    max_value: f64,
}

impl DataProcessor {
    pub fn new(valid_categories: Vec<String>, min_value: f64, max_value: f64) -> Self {
        DataProcessor {
            valid_categories,
            min_value,
            max_value,
        }
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if record.value < self.min_value || record.value > self.max_value {
            return Err(ProcessingError::InvalidValue);
        }

        if record.timestamp <= 0 {
            return Err(ProcessingError::InvalidTimestamp);
        }

        if !self.valid_categories.contains(&record.category) {
            return Err(ProcessingError::CategoryNotFound);
        }

        Ok(())
    }

    pub fn transform_record(&self, record: &DataRecord) -> Result<DataRecord, ProcessingError> {
        self.validate_record(record)?;

        let transformed_value = if record.value > 100.0 {
            record.value * 0.9
        } else {
            record.value * 1.1
        };

        let normalized_category = record.category.to_uppercase();

        Ok(DataRecord {
            id: record.id,
            value: transformed_value,
            timestamp: record.timestamp,
            category: normalized_category,
        })
    }

    pub fn process_batch(
        &self,
        records: Vec<DataRecord>,
    ) -> Result<Vec<DataRecord>, ProcessingError> {
        let mut processed_records = Vec::with_capacity(records.len());

        for record in records {
            match self.transform_record(&record) {
                Ok(transformed) => processed_records.push(transformed),
                Err(e) => return Err(e),
            }
        }

        Ok(processed_records)
    }

    pub fn serialize_records(&self, records: &[DataRecord]) -> Result<String, ProcessingError> {
        serde_json::to_string(records)
            .map_err(|e| ProcessingError::SerializationError(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_record_processing() {
        let processor = DataProcessor::new(
            vec!["temperature".to_string(), "pressure".to_string()],
            0.0,
            1000.0,
        );

        let record = DataRecord {
            id: 1,
            value: 50.0,
            timestamp: 1625097600,
            category: "temperature".to_string(),
        };

        let result = processor.transform_record(&record);
        assert!(result.is_ok());

        let transformed = result.unwrap();
        assert_eq!(transformed.category, "TEMPERATURE");
        assert_eq!(transformed.value, 55.0);
    }

    #[test]
    fn test_invalid_value() {
        let processor = DataProcessor::new(
            vec!["temperature".to_string()],
            0.0,
            100.0,
        );

        let record = DataRecord {
            id: 1,
            value: 150.0,
            timestamp: 1625097600,
            category: "temperature".to_string(),
        };

        let result = processor.validate_record(&record);
        assert!(result.is_err());
    }
}