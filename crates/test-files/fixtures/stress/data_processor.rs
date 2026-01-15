
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug)]
pub enum ValidationError {
    InvalidId,
    EmptyValues,
    ValueOutOfRange(f64),
    MissingMetadata(String),
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::InvalidId => write!(f, "Invalid record ID"),
            ValidationError::EmptyValues => write!(f, "Record values cannot be empty"),
            ValidationError::ValueOutOfRange(val) => write!(f, "Value {} out of valid range", val),
            ValidationError::MissingMetadata(key) => write!(f, "Missing metadata key: {}", key),
        }
    }
}

impl Error for ValidationError {}

pub struct DataProcessor {
    min_value: f64,
    max_value: f64,
    required_metadata: Vec<String>,
}

impl DataProcessor {
    pub fn new(min_value: f64, max_value: f64, required_metadata: Vec<String>) -> Self {
        DataProcessor {
            min_value,
            max_value,
            required_metadata,
        }
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), ValidationError> {
        if record.id == 0 {
            return Err(ValidationError::InvalidId);
        }

        if record.values.is_empty() {
            return Err(ValidationError::EmptyValues);
        }

        for &value in &record.values {
            if value < self.min_value || value > self.max_value {
                return Err(ValidationError::ValueOutOfRange(value));
            }
        }

        for key in &self.required_metadata {
            if !record.metadata.contains_key(key) {
                return Err(ValidationError::MissingMetadata(key.clone()));
            }
        }

        Ok(())
    }

    pub fn normalize_values(&self, record: &mut DataRecord) {
        let min_val = record.values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max_val = record.values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        
        if max_val > min_val {
            for value in &mut record.values {
                *value = (*value - min_val) / (max_val - min_val);
            }
        }
    }

    pub fn process_records(&self, records: &mut [DataRecord]) -> Vec<Result<(), ValidationError>> {
        let mut results = Vec::new();
        
        for record in records {
            match self.validate_record(record) {
                Ok(_) => {
                    self.normalize_values(record);
                    results.push(Ok(()));
                }
                Err(e) => {
                    results.push(Err(e));
                }
            }
        }
        
        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_record() -> DataRecord {
        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), "test".to_string());
        metadata.insert("version".to_string(), "1.0".to_string());
        
        DataRecord {
            id: 1,
            values: vec![10.0, 20.0, 30.0],
            metadata,
        }
    }

    #[test]
    fn test_valid_record() {
        let processor = DataProcessor::new(
            0.0,
            100.0,
            vec!["source".to_string(), "version".to_string()]
        );
        
        let record = create_test_record();
        assert!(processor.validate_record(&record).is_ok());
    }

    #[test]
    fn test_invalid_id() {
        let processor = DataProcessor::new(
            0.0,
            100.0,
            vec!["source".to_string()]
        );
        
        let mut record = create_test_record();
        record.id = 0;
        
        assert!(matches!(
            processor.validate_record(&record),
            Err(ValidationError::InvalidId)
        ));
    }

    #[test]
    fn test_normalize_values() {
        let processor = DataProcessor::new(
            0.0,
            100.0,
            vec!["source".to_string()]
        );
        
        let mut record = create_test_record();
        processor.normalize_values(&mut record);
        
        assert_eq!(record.values[0], 0.0);
        assert_eq!(record.values[1], 0.5);
        assert_eq!(record.values[2], 1.0);
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
            ValidationError::InvalidId => write!(f, "ID must be greater than zero"),
            ValidationError::EmptyName => write!(f, "Name cannot be empty"),
            ValidationError::NegativeValue => write!(f, "Value cannot be negative"),
            ValidationError::UnknownCategory => write!(f, "Category is not recognized"),
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
        let mut processor = DataProcessor {
            valid_categories: Vec::new(),
            transformation_rules: HashMap::new(),
        };
        
        processor.valid_categories.extend(vec![
            "standard".to_string(),
            "premium".to_string(),
            "economy".to_string(),
        ]);
        
        processor.transformation_rules.insert("standard".to_string(), 1.0);
        processor.transformation_rules.insert("premium".to_string(), 1.2);
        processor.transformation_rules.insert("economy".to_string(), 0.8);
        
        processor
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
            Some(multiplier) => record.value * multiplier,
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
    
    pub fn add_category(&mut self, category: String, multiplier: f64) {
        if !self.valid_categories.contains(&category) {
            self.valid_categories.push(category.clone());
            self.transformation_rules.insert(category, multiplier);
        }
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
            name: "Test Record".to_string(),
            value: 100.0,
            category: "standard".to_string(),
        };
        
        assert!(processor.validate_record(&record).is_ok());
        assert_eq!(processor.transform_value(&record), 100.0);
    }
    
    #[test]
    fn test_invalid_id() {
        let processor = DataProcessor::new();
        let record = DataRecord {
            id: 0,
            name: "Test Record".to_string(),
            value: 100.0,
            category: "standard".to_string(),
        };
        
        assert!(matches!(processor.validate_record(&record), Err(ValidationError::InvalidId)));
    }
    
    #[test]
    fn test_premium_multiplier() {
        let processor = DataProcessor::new();
        let record = DataRecord {
            id: 1,
            name: "Premium Item".to_string(),
            value: 100.0,
            category: "premium".to_string(),
        };
        
        assert!(processor.validate_record(&record).is_ok());
        assert_eq!(processor.transform_value(&record), 120.0);
    }
}