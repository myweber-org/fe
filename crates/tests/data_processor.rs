
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

#[derive(Debug)]
pub enum ValidationError {
    InvalidId,
    EmptyName,
    NegativeValue,
    UnknownCategory,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::InvalidId => write!(f, "ID must be positive integer"),
            ValidationError::EmptyName => write!(f, "Name cannot be empty"),
            ValidationError::NegativeValue => write!(f, "Value must be non-negative"),
            ValidationError::UnknownCategory => write!(f, "Category not recognized"),
        }
    }
}

impl Error for ValidationError {}

pub struct DataProcessor {
    valid_categories: Vec<String>,
    transformation_rules: HashMap<String, f64>,
}

impl DataProcessor {
    pub fn new() -> Self {
        let mut transformation_rules = HashMap::new();
        transformation_rules.insert("standard".to_string(), 1.0);
        transformation_rules.insert("premium".to_string(), 1.5);
        transformation_rules.insert("economy".to_string(), 0.8);
        
        DataProcessor {
            valid_categories: vec![
                "standard".to_string(),
                "premium".to_string(),
                "economy".to_string(),
            ],
            transformation_rules,
        }
    }
    
    pub fn validate_record(&self, record: &DataRecord) -> Result<(), ValidationError> {
        if record.id == 0 {
            return Err(ValidationError::InvalidId);
        }
        
        if record.name.trim().is_empty() {
            return Err(ValidationError::EmptyName);
        }
        
        if record.value < 0.0 {
            return Err(ValidationError::NegativeValue);
        }
        
        if !self.valid_categories.contains(&record.category) {
            return Err(ValidationError::UnknownCategory);
        }
        
        Ok(())
    }
    
    pub fn transform_value(&self, record: &DataRecord) -> f64 {
        match self.transformation_rules.get(&record.category) {
            Some(factor) => record.value * factor,
            None => record.value,
        }
    }
    
    pub fn process_batch(&self, records: Vec<DataRecord>) -> Vec<Result<f64, ValidationError>> {
        records
            .iter()
            .map(|record| {
                self.validate_record(record)
                    .map(|_| self.transform_value(record))
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_valid_record() {
        let processor = DataProcessor::new();
        let record = DataRecord {
            id: 1,
            name: "Test Item".to_string(),
            value: 100.0,
            category: "standard".to_string(),
        };
        
        assert!(processor.validate_record(&record).is_ok());
        assert_eq!(processor.transform_value(&record), 100.0);
    }
    
    #[test]
    fn test_invalid_category() {
        let processor = DataProcessor::new();
        let record = DataRecord {
            id: 1,
            name: "Test Item".to_string(),
            value: 100.0,
            category: "invalid".to_string(),
        };
        
        assert!(matches!(
            processor.validate_record(&record),
            Err(ValidationError::UnknownCategory)
        ));
    }
    
    #[test]
    fn test_premium_transformation() {
        let processor = DataProcessor::new();
        let record = DataRecord {
            id: 2,
            name: "Premium Item".to_string(),
            value: 100.0,
            category: "premium".to_string(),
        };
        
        assert_eq!(processor.transform_value(&record), 150.0);
    }
}