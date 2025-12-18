
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct ValidationError {
    details: String,
}

impl ValidationError {
    fn new(msg: &str) -> ValidationError {
        ValidationError {
            details: msg.to_string(),
        }
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl Error for ValidationError {
    fn description(&self) -> &str {
        &self.details
    }
}

pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub timestamp: i64,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, timestamp: i64) -> Result<DataRecord, ValidationError> {
        if id == 0 {
            return Err(ValidationError::new("ID cannot be zero"));
        }
        if value.is_nan() || value.is_infinite() {
            return Err(ValidationError::new("Value must be a finite number"));
        }
        if timestamp < 0 {
            return Err(ValidationError::new("Timestamp cannot be negative"));
        }

        Ok(DataRecord {
            id,
            value,
            timestamp,
        })
    }

    pub fn transform(&self, multiplier: f64) -> Result<f64, ValidationError> {
        if multiplier <= 0.0 {
            return Err(ValidationError::new("Multiplier must be positive"));
        }
        Ok(self.value * multiplier)
    }
}

pub fn process_records(records: &[DataRecord]) -> Vec<Result<f64, ValidationError>> {
    records
        .iter()
        .map(|record| record.transform(2.5))
        .collect()
}

pub fn calculate_average(records: &[DataRecord]) -> Option<f64> {
    if records.is_empty() {
        return None;
    }

    let sum: f64 = records.iter().map(|r| r.value).sum();
    Some(sum / records.len() as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_record_creation() {
        let record = DataRecord::new(1, 42.5, 1234567890).unwrap();
        assert_eq!(record.id, 1);
        assert_eq!(record.value, 42.5);
        assert_eq!(record.timestamp, 1234567890);
    }

    #[test]
    fn test_invalid_id() {
        let result = DataRecord::new(0, 42.5, 1234567890);
        assert!(result.is_err());
    }

    #[test]
    fn test_transform_valid() {
        let record = DataRecord::new(1, 10.0, 1234567890).unwrap();
        let transformed = record.transform(3.0).unwrap();
        assert_eq!(transformed, 30.0);
    }

    #[test]
    fn test_calculate_average() {
        let records = vec![
            DataRecord::new(1, 10.0, 1000).unwrap(),
            DataRecord::new(2, 20.0, 2000).unwrap(),
            DataRecord::new(3, 30.0, 3000).unwrap(),
        ];
        let avg = calculate_average(&records).unwrap();
        assert_eq!(avg, 20.0);
    }
}