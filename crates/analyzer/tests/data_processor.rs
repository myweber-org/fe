
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    id: u32,
    name: String,
    value: f64,
    metadata: HashMap<String, String>,
}

#[derive(Debug)]
pub enum ProcessingError {
    InvalidData(String),
    ValidationFailed(String),
    TransformationError(String),
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessingError::InvalidData(msg) => write!(f, "Invalid data: {}", msg),
            ProcessingError::ValidationFailed(msg) => write!(f, "Validation failed: {}", msg),
            ProcessingError::TransformationError(msg) => write!(f, "Transformation error: {}", msg),
        }
    }
}

impl Error for ProcessingError {}

impl DataRecord {
    pub fn new(id: u32, name: String, value: f64) -> Self {
        Self {
            id,
            name,
            value,
            metadata: HashMap::new(),
        }
    }

    pub fn validate(&self) -> Result<(), ProcessingError> {
        if self.id == 0 {
            return Err(ProcessingError::ValidationFailed("ID cannot be zero".to_string()));
        }
        
        if self.name.trim().is_empty() {
            return Err(ProcessingError::ValidationFailed("Name cannot be empty".to_string()));
        }
        
        if self.value.is_nan() || self.value.is_infinite() {
            return Err(ProcessingError::ValidationFailed("Value must be a finite number".to_string()));
        }
        
        Ok(())
    }

    pub fn transform(&mut self, multiplier: f64) -> Result<(), ProcessingError> {
        if multiplier <= 0.0 {
            return Err(ProcessingError::TransformationError(
                "Multiplier must be positive".to_string()
            ));
        }
        
        self.value *= multiplier;
        self.metadata.insert(
            "transformation_applied".to_string(),
            format!("multiplied by {}", multiplier)
        );
        
        Ok(())
    }

    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }
}

pub fn process_records(records: &mut [DataRecord], multiplier: f64) -> Result<Vec<DataRecord>, ProcessingError> {
    let mut processed = Vec::with_capacity(records.len());
    
    for record in records {
        record.validate()?;
        let mut cloned = record.clone();
        cloned.transform(multiplier)?;
        processed.push(cloned);
    }
    
    Ok(processed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_validation() {
        let valid_record = DataRecord::new(1, "Test".to_string(), 42.0);
        assert!(valid_record.validate().is_ok());

        let invalid_record = DataRecord::new(0, "".to_string(), f64::NAN);
        assert!(invalid_record.validate().is_err());
    }

    #[test]
    fn test_record_transformation() {
        let mut record = DataRecord::new(1, "Test".to_string(), 10.0);
        assert!(record.transform(2.5).is_ok());
        assert_eq!(record.value, 25.0);
        
        let metadata = record.get_metadata("transformation_applied");
        assert!(metadata.is_some());
    }

    #[test]
    fn test_batch_processing() {
        let mut records = vec![
            DataRecord::new(1, "Alpha".to_string(), 10.0),
            DataRecord::new(2, "Beta".to_string(), 20.0),
        ];
        
        let result = process_records(&mut records, 3.0);
        assert!(result.is_ok());
        
        let processed = result.unwrap();
        assert_eq!(processed[0].value, 30.0);
        assert_eq!(processed[1].value, 60.0);
    }
}