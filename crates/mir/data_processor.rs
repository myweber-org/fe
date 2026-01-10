
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
        if value < 0.0 || value > 1000.0 {
            return Err(ValidationError::new("Value must be between 0 and 1000"));
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

        let transformed = self.value * multiplier;
        if transformed > 5000.0 {
            return Err(ValidationError::new("Transformed value exceeds maximum limit"));
        }

        Ok(transformed)
    }
}

pub fn process_records(records: &[DataRecord]) -> Vec<Result<f64, ValidationError>> {
    records
        .iter()
        .map(|record| record.transform(2.5))
        .collect()
}

pub fn filter_valid_results(
    results: &[Result<f64, ValidationError>],
) -> (Vec<f64>, Vec<ValidationError>) {
    let mut valid_values = Vec::new();
    let mut errors = Vec::new();

    for result in results {
        match result {
            Ok(value) => valid_values.push(*value),
            Err(e) => errors.push(ValidationError::new(&e.details)),
        }
    }

    (valid_values, errors)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_record_creation() {
        let record = DataRecord::new(1, 100.0, 1625097600);
        assert!(record.is_ok());
        let record = record.unwrap();
        assert_eq!(record.id, 1);
        assert_eq!(record.value, 100.0);
        assert_eq!(record.timestamp, 1625097600);
    }

    #[test]
    fn test_invalid_record_id() {
        let record = DataRecord::new(0, 100.0, 1625097600);
        assert!(record.is_err());
        assert_eq!(
            record.unwrap_err().to_string(),
            "ID cannot be zero"
        );
    }

    #[test]
    fn test_record_transformation() {
        let record = DataRecord::new(1, 100.0, 1625097600).unwrap();
        let result = record.transform(2.0);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 200.0);
    }

    #[test]
    fn test_invalid_transformation() {
        let record = DataRecord::new(1, 3000.0, 1625097600).unwrap();
        let result = record.transform(2.0);
        assert!(result.is_err());
    }
}