
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DataError {
    #[error("Invalid input data: {0}")]
    InvalidInput(String),
    #[error("Processing failed: {0}")]
    ProcessingFailed(String),
    #[error("Validation error: {0}")]
    ValidationError(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRecord {
    pub id: u64,
    pub timestamp: i64,
    pub values: HashMap<String, f64>,
    pub tags: Vec<String>,
}

impl DataRecord {
    pub fn new(id: u64, timestamp: i64) -> Self {
        Self {
            id,
            timestamp,
            values: HashMap::new(),
            tags: Vec::new(),
        }
    }

    pub fn add_value(&mut self, key: &str, value: f64) -> Result<(), DataError> {
        if !value.is_finite() {
            return Err(DataError::InvalidInput(
                "Value must be finite number".to_string(),
            ));
        }
        self.values.insert(key.to_string(), value);
        Ok(())
    }

    pub fn add_tag(&mut self, tag: &str) {
        if !self.tags.contains(&tag.to_string()) {
            self.tags.push(tag.to_string());
        }
    }

    pub fn validate(&self) -> Result<(), DataError> {
        if self.values.is_empty() {
            return Err(DataError::ValidationError(
                "Record must contain at least one value".to_string(),
            ));
        }

        if self.timestamp < 0 {
            return Err(DataError::ValidationError(
                "Timestamp cannot be negative".to_string(),
            ));
        }

        Ok(())
    }

    pub fn transform_values<F>(&mut self, transformer: F) -> Result<(), DataError>
    where
        F: Fn(f64) -> f64,
    {
        for value in self.values.values_mut() {
            let transformed = transformer(*value);
            if !transformed.is_finite() {
                return Err(DataError::ProcessingFailed(
                    "Transformation produced non-finite value".to_string(),
                ));
            }
            *value = transformed;
        }
        Ok(())
    }
}

pub fn process_records(records: &mut [DataRecord]) -> Result<Vec<DataRecord>, DataError> {
    let mut processed = Vec::with_capacity(records.len());

    for record in records.iter_mut() {
        record.validate()?;
        record.transform_values(|v| v * 2.0)?;
        processed.push(record.clone());
    }

    Ok(processed)
}

pub fn calculate_statistics(records: &[DataRecord]) -> HashMap<String, (f64, f64, f64)> {
    let mut stats = HashMap::new();

    for record in records {
        for (key, value) in &record.values {
            let entry = stats.entry(key.clone()).or_insert((0.0, 0.0, 0.0));
            entry.0 += value;
            entry.1 = entry.1.max(*value);
            entry.2 = if entry.2 == 0.0 {
                *value
            } else {
                entry.2.min(*value)
            };
        }
    }

    stats
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_record_creation() {
        let record = DataRecord::new(1, 1234567890);
        assert_eq!(record.id, 1);
        assert_eq!(record.timestamp, 1234567890);
        assert!(record.values.is_empty());
        assert!(record.tags.is_empty());
    }

    #[test]
    fn test_add_valid_value() {
        let mut record = DataRecord::new(1, 1234567890);
        assert!(record.add_value("temperature", 25.5).is_ok());
        assert_eq!(record.values.get("temperature"), Some(&25.5));
    }

    #[test]
    fn test_add_invalid_value() {
        let mut record = DataRecord::new(1, 1234567890);
        let result = record.add_value("invalid", f64::INFINITY);
        assert!(result.is_err());
    }

    #[test]
    fn test_record_validation() {
        let mut record = DataRecord::new(1, 1234567890);
        assert!(record.validate().is_err());

        record.add_value("test", 1.0).unwrap();
        assert!(record.validate().is_ok());
    }

    #[test]
    fn test_value_transformation() {
        let mut record = DataRecord::new(1, 1234567890);
        record.add_value("value", 10.0).unwrap();
        record.transform_values(|v| v * 2.0).unwrap();
        assert_eq!(record.values.get("value"), Some(&20.0));
    }

    #[test]
    fn test_process_records() {
        let mut records = vec![
            DataRecord::new(1, 1000),
            DataRecord::new(2, 2000),
        ];

        records[0].add_value("a", 1.0).unwrap();
        records[1].add_value("b", 2.0).unwrap();

        let processed = process_records(&mut records).unwrap();
        assert_eq!(processed.len(), 2);
        assert_eq!(processed[0].values.get("a"), Some(&2.0));
        assert_eq!(processed[1].values.get("b"), Some(&4.0));
    }
}