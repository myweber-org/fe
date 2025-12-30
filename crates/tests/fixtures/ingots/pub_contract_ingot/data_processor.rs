
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub timestamp: u64,
}

#[derive(Debug)]
pub enum ValidationError {
    InvalidId,
    InvalidValue,
    InvalidTimestamp,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::InvalidId => write!(f, "Invalid record ID"),
            ValidationError::InvalidValue => write!(f, "Invalid value field"),
            ValidationError::InvalidTimestamp => write!(f, "Invalid timestamp"),
        }
    }
}

impl Error for ValidationError {}

impl DataRecord {
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.id == 0 {
            return Err(ValidationError::InvalidId);
        }
        
        if self.value.is_nan() || self.value.is_infinite() {
            return Err(ValidationError::InvalidValue);
        }
        
        if self.timestamp == 0 {
            return Err(ValidationError::InvalidTimestamp);
        }
        
        Ok(())
    }
    
    pub fn transform(&mut self, multiplier: f64) -> Result<(), ValidationError> {
        self.validate()?;
        self.value *= multiplier;
        Ok(())
    }
}

pub fn process_records(records: &mut [DataRecord], multiplier: f64) -> Result<usize, ValidationError> {
    let mut processed_count = 0;
    
    for record in records.iter_mut() {
        match record.transform(multiplier) {
            Ok(_) => processed_count += 1,
            Err(e) => return Err(e),
        }
    }
    
    Ok(processed_count)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_valid_record() {
        let mut record = DataRecord {
            id: 1,
            value: 42.5,
            timestamp: 1234567890,
        };
        
        assert!(record.validate().is_ok());
        assert!(record.transform(2.0).is_ok());
        assert_eq!(record.value, 85.0);
    }
    
    #[test]
    fn test_invalid_id() {
        let record = DataRecord {
            id: 0,
            value: 42.5,
            timestamp: 1234567890,
        };
        
        assert!(record.validate().is_err());
    }
    
    #[test]
    fn test_process_multiple_records() {
        let mut records = vec![
            DataRecord { id: 1, value: 10.0, timestamp: 1000 },
            DataRecord { id: 2, value: 20.0, timestamp: 2000 },
            DataRecord { id: 3, value: 30.0, timestamp: 3000 },
        ];
        
        let result = process_records(&mut records, 3.0);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 3);
        assert_eq!(records[0].value, 30.0);
        assert_eq!(records[1].value, 60.0);
        assert_eq!(records[2].value, 90.0);
    }
}