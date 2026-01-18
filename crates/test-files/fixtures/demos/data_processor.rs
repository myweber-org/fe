
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
                    "Value for key '{}' must be finite",
                    key
                )));
            }
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

    pub fn contains_tag(&self, tag: &str) -> bool {
        self.tags.iter().any(|t| t == tag)
    }
}

pub fn process_records(records: &mut [DataRecord]) -> Result<Vec<DataRecord>, DataError> {
    let mut processed = Vec::with_capacity(records.len());

    for record in records {
        record.validate()?;
        let mut processed_record = record.clone();
        processed_record.normalize_values();
        processed.push(processed_record);
    }

    Ok(processed)
}

pub fn filter_by_tag(records: &[DataRecord], tag: &str) -> Vec<DataRecord> {
    records
        .iter()
        .filter(|r| r.contains_tag(tag))
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_validation() {
        let mut record = DataRecord::new(1, 1234567890);
        record.add_value("temperature", 25.5);
        assert!(record.validate().is_ok());

        let invalid_record = DataRecord::new(0, 1234567890);
        assert!(invalid_record.validate().is_err());
    }

    #[test]
    fn test_normalize_values() {
        let mut record = DataRecord::new(1, 1234567890);
        record.add_value("a", 10.0);
        record.add_value("b", 20.0);
        record.add_value("c", 30.0);

        record.normalize_values();

        let expected_sum = 1.0;
        let actual_sum: f64 = record.values.values().sum();
        assert!((actual_sum - expected_sum).abs() < f64::EPSILON);
    }

    #[test]
    fn test_filter_by_tag() {
        let mut record1 = DataRecord::new(1, 1234567890);
        record1.add_tag("important");

        let mut record2 = DataRecord::new(2, 1234567891);
        record2.add_tag("normal");

        let records = vec![record1, record2];
        let filtered = filter_by_tag(&records, "important");

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, 1);
    }
}
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub category: String,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug)]
pub enum ValidationError {
    InvalidId,
    EmptyName,
    NegativeValue,
    InvalidCategory,
    MissingMetadata(String),
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::InvalidId => write!(f, "ID must be greater than 0"),
            ValidationError::EmptyName => write!(f, "Name cannot be empty"),
            ValidationError::NegativeValue => write!(f, "Value cannot be negative"),
            ValidationError::InvalidCategory => write!(f, "Category must be one of: A, B, C, D"),
            ValidationError::MissingMetadata(key) => write!(f, "Missing metadata key: {}", key),
        }
    }
}

impl Error for ValidationError {}

pub fn validate_record(record: &DataRecord) -> Result<(), ValidationError> {
    if record.id == 0 {
        return Err(ValidationError::InvalidId);
    }
    
    if record.name.trim().is_empty() {
        return Err(ValidationError::EmptyName);
    }
    
    if record.value < 0.0 {
        return Err(ValidationError::NegativeValue);
    }
    
    let valid_categories = ["A", "B", "C", "D"];
    if !valid_categories.contains(&record.category.as_str()) {
        return Err(ValidationError::InvalidCategory);
    }
    
    if !record.metadata.contains_key("source") {
        return Err(ValidationError::MissingMetadata("source".to_string()));
    }
    
    Ok(())
}

pub fn transform_record(record: &DataRecord) -> DataRecord {
    let mut transformed = record.clone();
    
    transformed.name = record.name.to_uppercase();
    transformed.value = (record.value * 100.0).round() / 100.0;
    
    if transformed.category == "D" {
        transformed.value *= 1.1;
    }
    
    let mut new_metadata = record.metadata.clone();
    new_metadata.insert("processed_timestamp".to_string(), 
                       chrono::Utc::now().to_rfc3339());
    new_metadata.insert("original_value".to_string(), 
                       record.value.to_string());
    
    transformed.metadata = new_metadata;
    transformed
}

pub fn process_records(records: Vec<DataRecord>) -> Result<Vec<DataRecord>, ValidationError> {
    let mut processed_records = Vec::new();
    
    for record in records {
        validate_record(&record)?;
        let transformed = transform_record(&record);
        processed_records.push(transformed);
    }
    
    Ok(processed_records)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn create_test_record() -> DataRecord {
        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), "test".to_string());
        
        DataRecord {
            id: 1,
            name: "Test Record".to_string(),
            value: 42.5,
            category: "A".to_string(),
            metadata,
        }
    }
    
    #[test]
    fn test_valid_record_validation() {
        let record = create_test_record();
        assert!(validate_record(&record).is_ok());
    }
    
    #[test]
    fn test_invalid_id_validation() {
        let mut record = create_test_record();
        record.id = 0;
        assert!(matches!(validate_record(&record), Err(ValidationError::InvalidId)));
    }
    
    #[test]
    fn test_empty_name_validation() {
        let mut record = create_test_record();
        record.name = "   ".to_string();
        assert!(matches!(validate_record(&record), Err(ValidationError::EmptyName)));
    }
    
    #[test]
    fn test_negative_value_validation() {
        let mut record = create_test_record();
        record.value = -10.0;
        assert!(matches!(validate_record(&record), Err(ValidationError::NegativeValue)));
    }
    
    #[test]
    fn test_transform_record() {
        let record = create_test_record();
        let transformed = transform_record(&record);
        
        assert_eq!(transformed.name, "TEST RECORD");
        assert_eq!(transformed.value, 42.5);
        assert!(transformed.metadata.contains_key("processed_timestamp"));
        assert!(transformed.metadata.contains_key("original_value"));
    }
    
    #[test]
    fn test_process_records() {
        let records = vec![create_test_record()];
        let result = process_records(records);
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 1);
    }
}