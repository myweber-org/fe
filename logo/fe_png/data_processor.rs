
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub timestamp: i64,
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
            ValidationError::InvalidId => write!(f, "ID must be greater than 0"),
            ValidationError::InvalidValue => write!(f, "Value must be between 0.0 and 1000.0"),
            ValidationError::InvalidTimestamp => write!(f, "Timestamp must be non-negative"),
        }
    }
}

impl Error for ValidationError {}

pub fn validate_record(record: &DataRecord) -> Result<(), ValidationError> {
    if record.id == 0 {
        return Err(ValidationError::InvalidId);
    }
    
    if record.value < 0.0 || record.value > 1000.0 {
        return Err(ValidationError::InvalidValue);
    }
    
    if record.timestamp < 0 {
        return Err(ValidationError::InvalidTimestamp);
    }
    
    Ok(())
}

pub fn transform_value(record: &mut DataRecord, multiplier: f64) -> Result<(), ValidationError> {
    validate_record(record)?;
    
    let new_value = record.value * multiplier;
    if new_value < 0.0 || new_value > 1000.0 {
        return Err(ValidationError::InvalidValue);
    }
    
    record.value = new_value;
    Ok(())
}

pub fn process_records(records: &mut [DataRecord]) -> Vec<Result<(), ValidationError>> {
    records
        .iter_mut()
        .map(|record| transform_value(record, 1.5))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_record() {
        let record = DataRecord {
            id: 1,
            value: 500.0,
            timestamp: 1234567890,
        };
        
        assert!(validate_record(&record).is_ok());
    }

    #[test]
    fn test_invalid_id() {
        let record = DataRecord {
            id: 0,
            value: 500.0,
            timestamp: 1234567890,
        };
        
        assert!(matches!(validate_record(&record), Err(ValidationError::InvalidId)));
    }

    #[test]
    fn test_transform_value() {
        let mut record = DataRecord {
            id: 1,
            value: 200.0,
            timestamp: 1234567890,
        };
        
        assert!(transform_value(&mut record, 2.0).is_ok());
        assert_eq!(record.value, 400.0);
    }

    #[test]
    fn test_process_records() {
        let mut records = vec![
            DataRecord { id: 1, value: 100.0, timestamp: 1000 },
            DataRecord { id: 2, value: 200.0, timestamp: 2000 },
            DataRecord { id: 0, value: 300.0, timestamp: 3000 },
        ];
        
        let results = process_records(&mut records);
        
        assert!(results[0].is_ok());
        assert!(results[1].is_ok());
        assert!(results[2].is_err());
        
        assert_eq!(records[0].value, 150.0);
        assert_eq!(records[1].value, 300.0);
    }
}