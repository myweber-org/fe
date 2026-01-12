
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRecord {
    pub id: u64,
    pub value: f64,
    pub timestamp: i64,
}

#[derive(Debug)]
pub enum ProcessingError {
    InvalidValue(f64),
    InvalidTimestamp(i64),
    ValidationFailed(String),
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessingError::InvalidValue(v) => write!(f, "Invalid value: {}", v),
            ProcessingError::InvalidTimestamp(t) => write!(f, "Invalid timestamp: {}", t),
            ProcessingError::ValidationFailed(msg) => write!(f, "Validation failed: {}", msg),
        }
    }
}

impl Error for ProcessingError {}

pub struct DataProcessor {
    min_value: f64,
    max_value: f64,
}

impl DataProcessor {
    pub fn new(min_value: f64, max_value: f64) -> Self {
        DataProcessor { min_value, max_value }
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if record.value < self.min_value || record.value > self.max_value {
            return Err(ProcessingError::InvalidValue(record.value));
        }

        if record.timestamp < 0 {
            return Err(ProcessingError::InvalidTimestamp(record.timestamp));
        }

        Ok(())
    }

    pub fn normalize_value(&self, record: &DataRecord) -> f64 {
        (record.value - self.min_value) / (self.max_value - self.min_value)
    }

    pub fn process_batch(&self, records: Vec<DataRecord>) -> Result<Vec<f64>, ProcessingError> {
        let mut results = Vec::with_capacity(records.len());

        for record in records {
            self.validate_record(&record)?;
            let normalized = self.normalize_value(&record);
            results.push(normalized);
        }

        if results.is_empty() {
            return Err(ProcessingError::ValidationFailed(
                "Empty batch provided".to_string(),
            ));
        }

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_valid_record() {
        let processor = DataProcessor::new(0.0, 100.0);
        let record = DataRecord {
            id: 1,
            value: 50.0,
            timestamp: 1625097600,
        };

        assert!(processor.validate_record(&record).is_ok());
    }

    #[test]
    fn test_validate_invalid_value() {
        let processor = DataProcessor::new(0.0, 100.0);
        let record = DataRecord {
            id: 1,
            value: 150.0,
            timestamp: 1625097600,
        };

        assert!(processor.validate_record(&record).is_err());
    }

    #[test]
    fn test_normalize_value() {
        let processor = DataProcessor::new(0.0, 100.0);
        let record = DataRecord {
            id: 1,
            value: 75.0,
            timestamp: 1625097600,
        };

        assert_eq!(processor.normalize_value(&record), 0.75);
    }

    #[test]
    fn test_process_batch() {
        let processor = DataProcessor::new(0.0, 100.0);
        let records = vec![
            DataRecord {
                id: 1,
                value: 25.0,
                timestamp: 1625097600,
            },
            DataRecord {
                id: 2,
                value: 75.0,
                timestamp: 1625097601,
            },
        ];

        let result = processor.process_batch(records);
        assert!(result.is_ok());
        let normalized_values = result.unwrap();
        assert_eq!(normalized_values, vec![0.25, 0.75]);
    }
}