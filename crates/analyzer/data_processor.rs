
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DataError {
    #[error("Invalid data format")]
    InvalidFormat,
    #[error("Missing required field: {0}")]
    MissingField(String),
    #[error("Validation failed: {0}")]
    ValidationFailed(String),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
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

    pub fn add_value(&mut self, key: &str, value: f64) {
        self.values.insert(key.to_string(), value);
    }

    pub fn add_tag(&mut self, tag: &str) {
        self.tags.push(tag.to_string());
    }

    pub fn validate(&self) -> Result<(), DataError> {
        if self.id == 0 {
            return Err(DataError::ValidationFailed("ID cannot be zero".to_string()));
        }

        if self.timestamp < 0 {
            return Err(DataError::ValidationFailed(
                "Timestamp cannot be negative".to_string(),
            ));
        }

        if self.values.is_empty() {
            return Err(DataError::ValidationFailed(
                "Record must contain at least one value".to_string(),
            ));
        }

        for (key, value) in &self.values {
            if key.trim().is_empty() {
                return Err(DataError::ValidationFailed(
                    "Value key cannot be empty".to_string(),
                ));
            }
            if !value.is_finite() {
                return Err(DataError::ValidationFailed(format!(
                    "Value for '{}' must be finite",
                    key
                )));
            }
        }

        Ok(())
    }

    pub fn transform(&mut self, multiplier: f64) {
        for value in self.values.values_mut() {
            *value *= multiplier;
        }
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.values.is_empty() {
            return None;
        }

        let sum: f64 = self.values.values().sum();
        Some(sum / self.values.len() as f64)
    }
}

pub fn process_records(records: &mut [DataRecord], multiplier: f64) -> Result<Vec<DataRecord>, DataError> {
    let mut processed = Vec::with_capacity(records.len());

    for record in records {
        record.validate()?;
        let mut processed_record = record.clone();
        processed_record.transform(multiplier);
        processed.push(processed_record);
    }

    Ok(processed)
}

pub fn filter_records_by_tag(records: &[DataRecord], tag: &str) -> Vec<DataRecord> {
    records
        .iter()
        .filter(|r| r.tags.contains(&tag.to_string()))
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_validation() {
        let mut valid_record = DataRecord::new(1, 1625097600);
        valid_record.add_value("temperature", 25.5);
        valid_record.add_tag("sensor");

        assert!(valid_record.validate().is_ok());

        let mut invalid_record = DataRecord::new(0, 1625097600);
        invalid_record.add_value("temperature", 25.5);
        assert!(invalid_record.validate().is_err());
    }

    #[test]
    fn test_record_transformation() {
        let mut record = DataRecord::new(1, 1625097600);
        record.add_value("value1", 10.0);
        record.add_value("value2", 20.0);

        record.transform(2.0);

        assert_eq!(record.values.get("value1"), Some(&20.0));
        assert_eq!(record.values.get("value2"), Some(&40.0));
    }

    #[test]
    fn test_average_calculation() {
        let mut record = DataRecord::new(1, 1625097600);
        record.add_value("a", 10.0);
        record.add_value("b", 20.0);
        record.add_value("c", 30.0);

        assert_eq!(record.calculate_average(), Some(20.0));
    }

    #[test]
    fn test_filter_by_tag() {
        let mut record1 = DataRecord::new(1, 1625097600);
        record1.add_tag("important");
        record1.add_value("data", 1.0);

        let mut record2 = DataRecord::new(2, 1625097601);
        record2.add_tag("normal");
        record2.add_value("data", 2.0);

        let records = vec![record1, record2];
        let filtered = filter_records_by_tag(&records, "important");

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, 1);
    }
}