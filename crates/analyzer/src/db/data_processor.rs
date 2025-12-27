
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug)]
pub enum ValidationError {
    InvalidId,
    EmptyName,
    NegativeValue,
    MissingMetadata(String),
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::InvalidId => write!(f, "ID must be greater than zero"),
            ValidationError::EmptyName => write!(f, "Name cannot be empty"),
            ValidationError::NegativeValue => write!(f, "Value cannot be negative"),
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
    
    if !record.metadata.contains_key("source") {
        return Err(ValidationError::MissingMetadata("source".to_string()));
    }
    
    Ok(())
}

pub fn transform_record(record: &DataRecord) -> DataRecord {
    let mut transformed = record.clone();
    transformed.name = record.name.to_uppercase();
    transformed.value = (record.value * 100.0).round() / 100.0;
    
    let mut new_metadata = record.metadata.clone();
    new_metadata.insert("processed".to_string(), "true".to_string());
    new_metadata.insert("original_value".to_string(), record.value.to_string());
    
    transformed.metadata = new_metadata;
    transformed
}

pub fn process_records(records: Vec<DataRecord>) -> Result<Vec<DataRecord>, ValidationError> {
    let mut processed_records = Vec::with_capacity(records.len());
    
    for record in records {
        validate_record(&record)?;
        let processed = transform_record(&record);
        processed_records.push(processed);
    }
    
    Ok(processed_records)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_validate_valid_record() {
        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), "test".to_string());
        
        let record = DataRecord {
            id: 1,
            name: "Test Record".to_string(),
            value: 42.5,
            metadata,
        };
        
        assert!(validate_record(&record).is_ok());
    }
    
    #[test]
    fn test_validate_invalid_id() {
        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), "test".to_string());
        
        let record = DataRecord {
            id: 0,
            name: "Test Record".to_string(),
            value: 42.5,
            metadata,
        };
        
        assert!(matches!(validate_record(&record), Err(ValidationError::InvalidId)));
    }
    
    #[test]
    fn test_transform_record() {
        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), "test".to_string());
        
        let record = DataRecord {
            id: 1,
            name: "test record".to_string(),
            value: 42.56789,
            metadata,
        };
        
        let transformed = transform_record(&record);
        
        assert_eq!(transformed.name, "TEST RECORD");
        assert_eq!(transformed.value, 42.57);
        assert_eq!(transformed.metadata.get("processed"), Some(&"true".to_string()));
        assert_eq!(transformed.metadata.get("original_value"), Some(&"42.56789".to_string()));
    }
}