
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    id: u32,
    value: f64,
    timestamp: i64,
}

#[derive(Debug)]
pub enum DataError {
    InvalidId,
    InvalidValue,
    InvalidTimestamp,
    TransformationError(String),
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataError::InvalidId => write!(f, "Invalid record ID"),
            DataError::InvalidValue => write!(f, "Invalid data value"),
            DataError::InvalidTimestamp => write!(f, "Invalid timestamp"),
            DataError::TransformationError(msg) => write!(f, "Transformation error: {}", msg),
        }
    }
}

impl Error for DataError {}

impl DataRecord {
    pub fn new(id: u32, value: f64, timestamp: i64) -> Result<Self, DataError> {
        if id == 0 {
            return Err(DataError::InvalidId);
        }
        if !value.is_finite() {
            return Err(DataError::InvalidValue);
        }
        if timestamp < 0 {
            return Err(DataError::InvalidTimestamp);
        }

        Ok(Self {
            id,
            value,
            timestamp,
        })
    }

    pub fn transform(&self, multiplier: f64) -> Result<f64, DataError> {
        if !multiplier.is_finite() || multiplier == 0.0 {
            return Err(DataError::TransformationError(
                "Invalid multiplier".to_string(),
            ));
        }

        let result = self.value * multiplier;
        if result.is_nan() || result.is_infinite() {
            Err(DataError::TransformationError(
                "Result is not finite".to_string(),
            ))
        } else {
            Ok(result)
        }
    }

    pub fn normalize(&self, min: f64, max: f64) -> Result<f64, DataError> {
        if min >= max || !min.is_finite() || !max.is_finite() {
            return Err(DataError::TransformationError(
                "Invalid normalization range".to_string(),
            ));
        }

        let normalized = (self.value - min) / (max - min);
        if normalized.is_nan() || normalized.is_infinite() {
            Err(DataError::TransformationError(
                "Normalization failed".to_string(),
            ))
        } else {
            Ok(normalized)
        }
    }
}

pub fn process_records(records: &[DataRecord]) -> Vec<Result<f64, DataError>> {
    records
        .iter()
        .map(|record| record.transform(2.5))
        .collect()
}

pub fn filter_valid_records(records: &[DataRecord]) -> Vec<&DataRecord> {
    records
        .iter()
        .filter(|record| record.value > 0.0 && record.timestamp > 0)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_record_creation() {
        let record = DataRecord::new(1, 42.5, 1234567890);
        assert!(record.is_ok());
    }

    #[test]
    fn test_invalid_id() {
        let record = DataRecord::new(0, 42.5, 1234567890);
        assert!(matches!(record, Err(DataError::InvalidId)));
    }

    #[test]
    fn test_transform_valid() {
        let record = DataRecord::new(1, 10.0, 1234567890).unwrap();
        let result = record.transform(2.0);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 20.0);
    }

    #[test]
    fn test_normalize_valid() {
        let record = DataRecord::new(1, 75.0, 1234567890).unwrap();
        let result = record.normalize(50.0, 100.0);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0.5);
    }
}