
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
            ValidationError::NegativeValue => write!(f, "Value must be non-negative"),
            ValidationError::MissingMetadata(key) => write!(f, "Missing metadata key: {}", key),
        }
    }
}

impl Error for ValidationError {}

impl DataRecord {
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.id == 0 {
            return Err(ValidationError::InvalidId);
        }
        
        if self.name.trim().is_empty() {
            return Err(ValidationError::EmptyName);
        }
        
        if self.value < 0.0 {
            return Err(ValidationError::NegativeValue);
        }
        
        Ok(())
    }
    
    pub fn transform(&mut self, multiplier: f64) -> &mut Self {
        self.value *= multiplier;
        self
    }
    
    pub fn add_metadata(&mut self, key: String, value: String) -> &mut Self {
        self.metadata.insert(key, value);
        self
    }
}

pub fn process_records(records: &mut [DataRecord], multiplier: f64) -> Result<Vec<DataRecord>, ValidationError> {
    let mut processed = Vec::with_capacity(records.len());
    
    for record in records {
        record.validate()?;
        let mut cloned = record.clone();
        cloned.transform(multiplier);
        processed.push(cloned);
    }
    
    Ok(processed)
}

pub fn calculate_statistics(records: &[DataRecord]) -> (f64, f64, f64) {
    if records.is_empty() {
        return (0.0, 0.0, 0.0);
    }
    
    let sum: f64 = records.iter().map(|r| r.value).sum();
    let count = records.len() as f64;
    let mean = sum / count;
    
    let variance: f64 = records.iter()
        .map(|r| (r.value - mean).powi(2))
        .sum::<f64>() / count;
    
    let std_dev = variance.sqrt();
    
    (mean, variance, std_dev)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_valid_record() {
        let mut record = DataRecord {
            id: 1,
            name: "Test".to_string(),
            value: 100.0,
            metadata: HashMap::new(),
        };
        
        assert!(record.validate().is_ok());
        
        record.transform(2.0);
        assert_eq!(record.value, 200.0);
    }
    
    #[test]
    fn test_invalid_record() {
        let record = DataRecord {
            id: 0,
            name: "".to_string(),
            value: -10.0,
            metadata: HashMap::new(),
        };
        
        assert!(record.validate().is_err());
    }
    
    #[test]
    fn test_process_records() {
        let mut records = vec![
            DataRecord {
                id: 1,
                name: "A".to_string(),
                value: 10.0,
                metadata: HashMap::new(),
            },
            DataRecord {
                id: 2,
                name: "B".to_string(),
                value: 20.0,
                metadata: HashMap::new(),
            },
        ];
        
        let result = process_records(&mut records, 3.0);
        assert!(result.is_ok());
        
        let processed = result.unwrap();
        assert_eq!(processed[0].value, 30.0);
        assert_eq!(processed[1].value, 60.0);
    }
    
    #[test]
    fn test_statistics() {
        let records = vec![
            DataRecord {
                id: 1,
                name: "A".to_string(),
                value: 10.0,
                metadata: HashMap::new(),
            },
            DataRecord {
                id: 2,
                name: "B".to_string(),
                value: 20.0,
                metadata: HashMap::new(),
            },
            DataRecord {
                id: 3,
                name: "C".to_string(),
                value: 30.0,
                metadata: HashMap::new(),
            },
        ];
        
        let (mean, variance, std_dev) = calculate_statistics(&records);
        assert_eq!(mean, 20.0);
        assert_eq!(variance, 66.66666666666667);
        assert_eq!(std_dev, 8.16496580927726);
    }
}