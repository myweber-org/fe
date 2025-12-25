
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRecord {
    pub id: u64,
    pub value: f64,
    pub timestamp: i64,
    pub category: String,
}

#[derive(Debug)]
pub enum ProcessingError {
    InvalidValue(f64),
    InvalidTimestamp(i64),
    EmptyCategory,
    ValidationFailed(String),
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessingError::InvalidValue(v) => write!(f, "Invalid value: {}", v),
            ProcessingError::InvalidTimestamp(t) => write!(f, "Invalid timestamp: {}", t),
            ProcessingError::EmptyCategory => write!(f, "Category cannot be empty"),
            ProcessingError::ValidationFailed(msg) => write!(f, "Validation failed: {}", msg),
        }
    }
}

impl Error for ProcessingError {}

pub struct DataProcessor {
    validation_enabled: bool,
    max_value_threshold: f64,
}

impl DataProcessor {
    pub fn new(validation_enabled: bool, max_value_threshold: f64) -> Self {
        Self {
            validation_enabled,
            max_value_threshold,
        }
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if !self.validation_enabled {
            return Ok(());
        }

        if record.value.is_nan() || record.value.is_infinite() {
            return Err(ProcessingError::InvalidValue(record.value));
        }

        if record.value.abs() > self.max_value_threshold {
            return Err(ProcessingError::InvalidValue(record.value));
        }

        if record.timestamp < 0 {
            return Err(ProcessingError::InvalidTimestamp(record.timestamp));
        }

        if record.category.trim().is_empty() {
            return Err(ProcessingError::EmptyCategory);
        }

        Ok(())
    }

    pub fn process_record(&self, mut record: DataRecord) -> Result<DataRecord, ProcessingError> {
        self.validate_record(&record)?;

        record.value = self.normalize_value(record.value);
        record.category = self.standardize_category(&record.category);
        
        Ok(record)
    }

    pub fn process_batch(&self, records: Vec<DataRecord>) -> Vec<Result<DataRecord, ProcessingError>> {
        records
            .into_iter()
            .map(|record| self.process_record(record))
            .collect()
    }

    fn normalize_value(&self, value: f64) -> f64 {
        if value >= 0.0 {
            value.ln_1p()
        } else {
            -((-value).ln_1p())
        }
    }

    fn standardize_category(&self, category: &str) -> String {
        let mut standardized = category.trim().to_lowercase();
        if let Some(first_char) = standardized.chars().next() {
            standardized = first_char.to_uppercase().to_string() + &standardized[1..];
        }
        standardized
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_valid_record() {
        let processor = DataProcessor::new(true, 1000.0);
        let record = DataRecord {
            id: 1,
            value: 42.5,
            timestamp: 1234567890,
            category: "Temperature".to_string(),
        };
        
        assert!(processor.validate_record(&record).is_ok());
    }

    #[test]
    fn test_validate_invalid_value() {
        let processor = DataProcessor::new(true, 100.0);
        let record = DataRecord {
            id: 1,
            value: 150.0,
            timestamp: 1234567890,
            category: "Test".to_string(),
        };
        
        assert!(processor.validate_record(&record).is_err());
    }

    #[test]
    fn test_process_record_normalization() {
        let processor = DataProcessor::new(true, 1000.0);
        let record = DataRecord {
            id: 1,
            value: 10.0,
            timestamp: 1234567890,
            category: "  pressure  ".to_string(),
        };
        
        let processed = processor.process_record(record).unwrap();
        assert!((processed.value - 2.397895).abs() < 0.0001);
        assert_eq!(processed.category, "Pressure");
    }

    #[test]
    fn test_batch_processing() {
        let processor = DataProcessor::new(true, 1000.0);
        let records = vec![
            DataRecord {
                id: 1,
                value: 10.0,
                timestamp: 1234567890,
                category: "A".to_string(),
            },
            DataRecord {
                id: 2,
                value: f64::NAN,
                timestamp: 1234567890,
                category: "B".to_string(),
            },
        ];
        
        let results = processor.process_batch(records);
        assert_eq!(results.len(), 2);
        assert!(results[0].is_ok());
        assert!(results[1].is_err());
    }
}