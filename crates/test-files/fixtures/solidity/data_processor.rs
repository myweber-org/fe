
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Serialize, Deserialize)]
pub struct DataRecord {
    id: u32,
    value: f64,
    timestamp: i64,
}

#[derive(Debug, Error)]
pub enum ProcessingError {
    #[error("Invalid data value: {0}")]
    InvalidValue(f64),
    #[error("Timestamp out of range: {0}")]
    InvalidTimestamp(i64),
    #[error("ID must be positive: {0}")]
    InvalidId(u32),
}

impl DataRecord {
    pub fn validate(&self) -> Result<(), ProcessingError> {
        if self.id == 0 {
            return Err(ProcessingError::InvalidId(self.id));
        }
        
        if !self.value.is_finite() {
            return Err(ProcessingError::InvalidValue(self.value));
        }
        
        if self.timestamp < 0 {
            return Err(ProcessingError::InvalidTimestamp(self.timestamp));
        }
        
        Ok(())
    }
    
    pub fn transform(&mut self, multiplier: f64) -> Result<(), ProcessingError> {
        self.validate()?;
        
        if multiplier.is_finite() && multiplier != 0.0 {
            self.value *= multiplier;
            Ok(())
        } else {
            Err(ProcessingError::InvalidValue(multiplier))
        }
    }
}

pub fn process_records(records: &mut [DataRecord], multiplier: f64) -> Vec<Result<(), ProcessingError>> {
    records
        .iter_mut()
        .map(|record| record.transform(multiplier))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_valid_record() {
        let mut record = DataRecord {
            id: 1,
            value: 42.5,
            timestamp: 1633046400,
        };
        
        assert!(record.validate().is_ok());
        assert!(record.transform(2.0).is_ok());
        assert_eq!(record.value, 85.0);
    }
    
    #[test]
    fn test_invalid_id() {
        let record = DataRecord {
            id: 0,
            value: 10.0,
            timestamp: 1633046400,
        };
        
        assert!(matches!(record.validate(), Err(ProcessingError::InvalidId(0))));
    }
}