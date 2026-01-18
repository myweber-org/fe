
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub category: String,
}

#[derive(Debug)]
pub enum ValidationError {
    InvalidId,
    InvalidValue,
    EmptyCategory,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::InvalidId => write!(f, "ID must be greater than 0"),
            ValidationError::InvalidValue => write!(f, "Value must be between 0.0 and 1000.0"),
            ValidationError::EmptyCategory => write!(f, "Category cannot be empty"),
        }
    }
}

impl Error for ValidationError {}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: String) -> Result<Self, ValidationError> {
        if id == 0 {
            return Err(ValidationError::InvalidId);
        }
        
        if value < 0.0 || value > 1000.0 {
            return Err(ValidationError::InvalidValue);
        }
        
        if category.trim().is_empty() {
            return Err(ValidationError::EmptyCategory);
        }
        
        Ok(Self {
            id,
            value,
            category: category.trim().to_string(),
        })
    }
    
    pub fn normalize_value(&mut self) {
        self.value = (self.value * 100.0).round() / 100.0;
    }
    
    pub fn to_json(&self) -> String {
        format!(
            r#"{{"id":{},"value":{},"category":"{}"}}"#,
            self.id, self.value, self.category
        )
    }
}

pub fn process_records(records: &mut [DataRecord]) -> Vec<String> {
    records.iter_mut().for_each(|record| {
        record.normalize_value();
    });
    
    records
        .iter()
        .map(|record| record.to_json())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_valid_record_creation() {
        let record = DataRecord::new(1, 50.5, "test".to_string());
        assert!(record.is_ok());
        
        let record = record.unwrap();
        assert_eq!(record.id, 1);
        assert_eq!(record.value, 50.5);
        assert_eq!(record.category, "test");
    }
    
    #[test]
    fn test_invalid_id() {
        let record = DataRecord::new(0, 50.5, "test".to_string());
        assert!(matches!(record, Err(ValidationError::InvalidId)));
    }
    
    #[test]
    fn test_normalize_value() {
        let mut record = DataRecord::new(1, 50.555, "test".to_string()).unwrap();
        record.normalize_value();
        assert_eq!(record.value, 50.56);
    }
    
    #[test]
    fn test_to_json() {
        let record = DataRecord::new(1, 50.5, "test".to_string()).unwrap();
        let json = record.to_json();
        assert_eq!(json, r#"{"id":1,"value":50.5,"category":"test"}"#);
    }
}