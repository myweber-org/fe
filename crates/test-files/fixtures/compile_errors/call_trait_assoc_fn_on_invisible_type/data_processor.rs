
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub timestamp: i64,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug)]
pub enum ValidationError {
    InvalidId,
    InvalidTimestamp,
    EmptyValues,
    MetadataKeyTooLong,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ValidationError::InvalidId => write!(f, "ID must be greater than zero"),
            ValidationError::InvalidTimestamp => write!(f, "Timestamp cannot be negative"),
            ValidationError::EmptyValues => write!(f, "Values array cannot be empty"),
            ValidationError::MetadataKeyTooLong => write!(f, "Metadata key exceeds maximum length"),
        }
    }
}

impl Error for ValidationError {}

impl DataRecord {
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.id == 0 {
            return Err(ValidationError::InvalidId);
        }
        
        if self.timestamp < 0 {
            return Err(ValidationError::InvalidTimestamp);
        }
        
        if self.values.is_empty() {
            return Err(ValidationError::EmptyValues);
        }
        
        for key in self.metadata.keys() {
            if key.len() > 50 {
                return Err(ValidationError::MetadataKeyTooLong);
            }
        }
        
        Ok(())
    }
    
    pub fn transform_values(&mut self, transformer: fn(f64) -> f64) {
        self.values = self.values.iter().map(|&v| transformer(v)).collect();
    }
    
    pub fn calculate_statistics(&self) -> (f64, f64, f64) {
        let count = self.values.len() as f64;
        let sum: f64 = self.values.iter().sum();
        let mean = sum / count;
        
        let variance: f64 = self.values.iter()
            .map(|&v| (v - mean).powi(2))
            .sum::<f64>() / count;
        
        let std_dev = variance.sqrt();
        
        (mean, variance, std_dev)
    }
}

pub fn normalize_value(value: f64, min: f64, max: f64) -> f64 {
    if max == min {
        return 0.0;
    }
    (value - min) / (max - min)
}

pub fn process_records(records: &mut [DataRecord]) -> Result<Vec<DataRecord>, ValidationError> {
    let mut processed = Vec::with_capacity(records.len());
    
    for record in records {
        record.validate()?;
        let mut processed_record = record.clone();
        processed_record.transform_values(|v| v * 2.0);
        processed.push(processed_record);
    }
    
    Ok(processed)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_record_validation() {
        let valid_record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: vec![1.0, 2.0, 3.0],
            metadata: HashMap::new(),
        };
        
        assert!(valid_record.validate().is_ok());
        
        let invalid_record = DataRecord {
            id: 0,
            timestamp: 1234567890,
            values: vec![1.0, 2.0, 3.0],
            metadata: HashMap::new(),
        };
        
        assert!(invalid_record.validate().is_err());
    }
    
    #[test]
    fn test_statistics_calculation() {
        let record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: vec![1.0, 2.0, 3.0, 4.0, 5.0],
            metadata: HashMap::new(),
        };
        
        let (mean, variance, std_dev) = record.calculate_statistics();
        
        assert_eq!(mean, 3.0);
        assert_eq!(variance, 2.0);
        assert_eq!(std_dev, 2.0_f64.sqrt());
    }
}