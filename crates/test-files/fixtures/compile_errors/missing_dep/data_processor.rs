
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRecord {
    pub id: u64,
    pub timestamp: i64,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug)]
pub enum ProcessingError {
    InvalidData(String),
    TransformationFailed(String),
    ValidationError(String),
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessingError::InvalidData(msg) => write!(f, "Invalid data: {}", msg),
            ProcessingError::TransformationFailed(msg) => write!(f, "Transformation failed: {}", msg),
            ProcessingError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl Error for ProcessingError {}

pub struct DataProcessor {
    validation_threshold: f64,
    transformation_factor: f64,
}

impl DataProcessor {
    pub fn new(validation_threshold: f64, transformation_factor: f64) -> Self {
        DataProcessor {
            validation_threshold,
            transformation_factor,
        }
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if record.id == 0 {
            return Err(ProcessingError::ValidationError("ID cannot be zero".to_string()));
        }

        if record.timestamp < 0 {
            return Err(ProcessingError::ValidationError("Timestamp cannot be negative".to_string()));
        }

        if record.values.is_empty() {
            return Err(ProcessingError::ValidationError("Values cannot be empty".to_string()));
        }

        for value in &record.values {
            if value.is_nan() || value.is_infinite() {
                return Err(ProcessingError::ValidationError(
                    "Values contain NaN or infinite numbers".to_string(),
                ));
            }

            if value.abs() > self.validation_threshold {
                return Err(ProcessingError::ValidationError(format!(
                    "Value {} exceeds threshold {}",
                    value, self.validation_threshold
                )));
            }
        }

        Ok(())
    }

    pub fn transform_values(&self, record: &mut DataRecord) -> Result<(), ProcessingError> {
        if record.values.iter().any(|&v| v < 0.0) {
            return Err(ProcessingError::TransformationFailed(
                "Cannot transform negative values".to_string(),
            ));
        }

        for value in &mut record.values {
            *value = (*value * self.transformation_factor).ln_1p();
            
            if value.is_nan() || value.is_infinite() {
                return Err(ProcessingError::TransformationFailed(
                    "Transformation produced invalid result".to_string(),
                ));
            }
        }

        record.metadata.insert(
            "transformation_applied".to_string(),
            format!("factor_{}", self.transformation_factor),
        );

        Ok(())
    }

    pub fn process_batch(&self, records: &mut [DataRecord]) -> Vec<Result<DataRecord, ProcessingError>> {
        records
            .iter_mut()
            .map(|record| {
                self.validate_record(record)
                    .and_then(|_| self.transform_values(record))
                    .map(|_| record.clone())
            })
            .collect()
    }

    pub fn calculate_statistics(&self, records: &[DataRecord]) -> HashMap<String, f64> {
        let mut stats = HashMap::new();

        if records.is_empty() {
            return stats;
        }

        let total_values: usize = records.iter().map(|r| r.values.len()).sum();
        let all_values: Vec<f64> = records.iter().flat_map(|r| r.values.clone()).collect();

        stats.insert("record_count".to_string(), records.len() as f64);
        stats.insert("total_values".to_string(), total_values as f64);

        if !all_values.is_empty() {
            let sum: f64 = all_values.iter().sum();
            let mean = sum / all_values.len() as f64;
            let variance: f64 = all_values.iter().map(|&v| (v - mean).powi(2)).sum::<f64>() / all_values.len() as f64;
            let std_dev = variance.sqrt();

            stats.insert("mean".to_string(), mean);
            stats.insert("variance".to_string(), variance);
            stats.insert("std_dev".to_string(), std_dev);
            stats.insert("min".to_string(), *all_values.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap());
            stats.insert("max".to_string(), *all_values.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap());
        }

        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_record() -> DataRecord {
        DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: vec![1.0, 2.0, 3.0],
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn test_validation_success() {
        let processor = DataProcessor::new(100.0, 2.0);
        let record = create_test_record();
        assert!(processor.validate_record(&record).is_ok());
    }

    #[test]
    fn test_validation_failure() {
        let processor = DataProcessor::new(2.0, 2.0);
        let record = create_test_record();
        assert!(processor.validate_record(&record).is_err());
    }

    #[test]
    fn test_transformation() {
        let processor = DataProcessor::new(100.0, 2.0);
        let mut record = create_test_record();
        assert!(processor.transform_values(&mut record).is_ok());
        assert!(record.metadata.contains_key("transformation_applied"));
    }

    #[test]
    fn test_batch_processing() {
        let processor = DataProcessor::new(100.0, 2.0);
        let mut records = vec![create_test_record(), create_test_record()];
        let results = processor.process_batch(&mut records);
        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|r| r.is_ok()));
    }

    #[test]
    fn test_statistics_calculation() {
        let processor = DataProcessor::new(100.0, 2.0);
        let records = vec![create_test_record(), create_test_record()];
        let stats = processor.calculate_statistics(&records);
        assert!(stats.contains_key("mean"));
        assert!(stats.contains_key("std_dev"));
        assert_eq!(stats["record_count"], 2.0);
    }
}