use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub timestamp: i64,
}

#[derive(Debug)]
pub enum ProcessingError {
    InvalidValue(f64),
    InvalidTimestamp(i64),
    MissingField(String),
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessingError::InvalidValue(v) => write!(f, "Invalid value: {}", v),
            ProcessingError::InvalidTimestamp(t) => write!(f, "Invalid timestamp: {}", t),
            ProcessingError::MissingField(field) => write!(f, "Missing field: {}", field),
        }
    }
}

impl Error for ProcessingError {}

pub fn validate_record(record: &DataRecord) -> Result<(), ProcessingError> {
    if record.value.is_nan() || record.value.is_infinite() {
        return Err(ProcessingError::InvalidValue(record.value));
    }
    
    if record.timestamp < 0 {
        return Err(ProcessingError::InvalidTimestamp(record.timestamp));
    }
    
    Ok(())
}

pub fn transform_record(record: &DataRecord) -> Result<DataRecord, ProcessingError> {
    validate_record(record)?;
    
    let transformed = DataRecord {
        id: record.id,
        value: (record.value * 100.0).round() / 100.0,
        timestamp: record.timestamp,
    };
    
    Ok(transformed)
}

pub fn process_records(records: Vec<DataRecord>) -> Vec<Result<DataRecord, ProcessingError>> {
    records
        .into_iter()
        .map(|record| transform_record(&record))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_record() {
        let record = DataRecord {
            id: 1,
            value: 42.5,
            timestamp: 1625097600,
        };
        
        assert!(validate_record(&record).is_ok());
    }

    #[test]
    fn test_invalid_value() {
        let record = DataRecord {
            id: 2,
            value: f64::NAN,
            timestamp: 1625097600,
        };
        
        assert!(validate_record(&record).is_err());
    }

    #[test]
    fn test_transform_record() {
        let record = DataRecord {
            id: 3,
            value: 123.456,
            timestamp: 1625097600,
        };
        
        let transformed = transform_record(&record).unwrap();
        assert_eq!(transformed.value, 123.46);
    }
}