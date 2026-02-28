
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
    min_value: f64,
    max_value: f64,
    allowed_categories: Vec<String>,
}

impl DataProcessor {
    pub fn new(min_value: f64, max_value: f64, allowed_categories: Vec<String>) -> Self {
        DataProcessor {
            min_value,
            max_value,
            allowed_categories,
        }
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if record.value < self.min_value || record.value > self.max_value {
            return Err(ProcessingError::InvalidValue(record.value));
        }

        if record.timestamp < 0 {
            return Err(ProcessingError::InvalidTimestamp(record.timestamp));
        }

        if record.category.trim().is_empty() {
            return Err(ProcessingError::EmptyCategory);
        }

        if !self.allowed_categories.contains(&record.category) {
            return Err(ProcessingError::ValidationFailed(format!(
                "Category '{}' not allowed",
                record.category
            )));
        }

        Ok(())
    }

    pub fn transform_record(&self, record: &DataRecord) -> DataRecord {
        let normalized_value = (record.value - self.min_value) / (self.max_value - self.min_value);
        
        DataRecord {
            id: record.id,
            value: normalized_value,
            timestamp: record.timestamp,
            category: record.category.to_uppercase(),
        }
    }

    pub fn process_batch(&self, records: Vec<DataRecord>) -> Result<Vec<DataRecord>, ProcessingError> {
        let mut processed_records = Vec::with_capacity(records.len());
        
        for record in records {
            self.validate_record(&record)?;
            let transformed = self.transform_record(&record);
            processed_records.push(transformed);
        }
        
        Ok(processed_records)
    }

    pub fn filter_by_category(&self, records: &[DataRecord], category: &str) -> Vec<DataRecord> {
        records
            .iter()
            .filter(|r| r.category == category)
            .cloned()
            .collect()
    }

    pub fn calculate_statistics(&self, records: &[DataRecord]) -> (f64, f64, f64) {
        if records.is_empty() {
            return (0.0, 0.0, 0.0);
        }

        let sum: f64 = records.iter().map(|r| r.value).sum();
        let count = records.len() as f64;
        let mean = sum / count;

        let variance: f64 = records
            .iter()
            .map(|r| (r.value - mean).powi(2))
            .sum::<f64>() / count;

        let std_dev = variance.sqrt();

        (mean, variance, std_dev)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_processor() -> DataProcessor {
        DataProcessor::new(
            0.0,
            100.0,
            vec!["A".to_string(), "B".to_string(), "C".to_string()],
        )
    }

    #[test]
    fn test_validate_valid_record() {
        let processor = create_test_processor();
        let record = DataRecord {
            id: 1,
            value: 50.0,
            timestamp: 1234567890,
            category: "A".to_string(),
        };

        assert!(processor.validate_record(&record).is_ok());
    }

    #[test]
    fn test_validate_invalid_value() {
        let processor = create_test_processor();
        let record = DataRecord {
            id: 1,
            value: 150.0,
            timestamp: 1234567890,
            category: "A".to_string(),
        };

        assert!(matches!(
            processor.validate_record(&record),
            Err(ProcessingError::InvalidValue(150.0))
        ));
    }

    #[test]
    fn test_transform_record() {
        let processor = create_test_processor();
        let record = DataRecord {
            id: 1,
            value: 50.0,
            timestamp: 1234567890,
            category: "a".to_string(),
        };

        let transformed = processor.transform_record(&record);
        assert_eq!(transformed.value, 0.5);
        assert_eq!(transformed.category, "A");
    }

    #[test]
    fn test_process_batch() {
        let processor = create_test_processor();
        let records = vec![
            DataRecord {
                id: 1,
                value: 25.0,
                timestamp: 1234567890,
                category: "A".to_string(),
            },
            DataRecord {
                id: 2,
                value: 75.0,
                timestamp: 1234567891,
                category: "B".to_string(),
            },
        ];

        let result = processor.process_batch(records);
        assert!(result.is_ok());
        let processed = result.unwrap();
        assert_eq!(processed.len(), 2);
        assert_eq!(processed[0].value, 0.25);
        assert_eq!(processed[1].value, 0.75);
    }

    #[test]
    fn test_filter_by_category() {
        let processor = create_test_processor();
        let records = vec![
            DataRecord {
                id: 1,
                value: 25.0,
                timestamp: 1234567890,
                category: "A".to_string(),
            },
            DataRecord {
                id: 2,
                value: 75.0,
                timestamp: 1234567891,
                category: "B".to_string(),
            },
            DataRecord {
                id: 3,
                value: 50.0,
                timestamp: 1234567892,
                category: "A".to_string(),
            },
        ];

        let filtered = processor.filter_by_category(&records, "A");
        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().all(|r| r.category == "A"));
    }

    #[test]
    fn test_calculate_statistics() {
        let processor = create_test_processor();
        let records = vec![
            DataRecord {
                id: 1,
                value: 10.0,
                timestamp: 1234567890,
                category: "A".to_string(),
            },
            DataRecord {
                id: 2,
                value: 20.0,
                timestamp: 1234567891,
                category: "A".to_string(),
            },
            DataRecord {
                id: 3,
                value: 30.0,
                timestamp: 1234567892,
                category: "A".to_string(),
            },
        ];

        let (mean, variance, std_dev) = processor.calculate_statistics(&records);
        assert_eq!(mean, 20.0);
        assert_eq!(variance, 66.66666666666667);
        assert_eq!(std_dev, 8.16496580927726);
    }
}