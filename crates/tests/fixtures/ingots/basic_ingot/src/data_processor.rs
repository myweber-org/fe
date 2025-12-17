
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
            ValidationError::InvalidId => write!(f, "ID must be greater than zero"),
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
    
    pub fn normalize_value(&mut self, factor: f64) {
        if factor != 0.0 {
            self.value /= factor;
        }
    }
    
    pub fn to_json(&self) -> String {
        format!(
            r#"{{"id":{},"value":{},"category":"{}"}}"#,
            self.id, self.value, self.category
        )
    }
}

pub fn process_records(records: &mut [DataRecord], normalization_factor: f64) -> Vec<String> {
    records
        .iter_mut()
        .map(|record| {
            record.normalize_value(normalization_factor);
            record.to_json()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_valid_record_creation() {
        let record = DataRecord::new(1, 500.0, "analytics".to_string());
        assert!(record.is_ok());
        
        let record = record.unwrap();
        assert_eq!(record.id, 1);
        assert_eq!(record.value, 500.0);
        assert_eq!(record.category, "analytics");
    }
    
    #[test]
    fn test_invalid_id() {
        let record = DataRecord::new(0, 500.0, "test".to_string());
        assert!(matches!(record, Err(ValidationError::InvalidId)));
    }
    
    #[test]
    fn test_normalize_value() {
        let mut record = DataRecord::new(1, 100.0, "data".to_string()).unwrap();
        record.normalize_value(10.0);
        assert_eq!(record.value, 10.0);
    }
    
    #[test]
    fn test_json_output() {
        let record = DataRecord::new(42, 123.45, "metrics".to_string()).unwrap();
        let json = record.to_json();
        assert!(json.contains("\"id\":42"));
        assert!(json.contains("\"value\":123.45"));
        assert!(json.contains("\"category\":\"metrics\""));
    }
}