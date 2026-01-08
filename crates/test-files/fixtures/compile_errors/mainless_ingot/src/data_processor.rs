
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRecord {
    pub id: u64,
    pub timestamp: i64,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

#[derive(Error, Debug)]
pub enum ProcessingError {
    #[error("Invalid data format")]
    InvalidFormat,
    #[error("Data validation failed: {0}")]
    ValidationFailed(String),
    #[error("Transformation error: {0}")]
    TransformationError(String),
}

pub struct DataProcessor {
    validation_threshold: f64,
    transformation_factor: f64,
}

impl DataProcessor {
    pub fn new(validation_threshold: f64, transformation_factor: f64) -> Self {
        Self {
            validation_threshold,
            transformation_factor,
        }
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if record.values.is_empty() {
            return Err(ProcessingError::ValidationFailed(
                "Empty values array".to_string(),
            ));
        }

        for value in &record.values {
            if value.is_nan() || value.is_infinite() {
                return Err(ProcessingError::ValidationFailed(
                    "Invalid numeric value".to_string(),
                ));
            }

            if value.abs() > self.validation_threshold {
                return Err(ProcessingError::ValidationFailed(format!(
                    "Value {} exceeds threshold {}",
                    value, self.validation_threshold
                )));
            }
        }

        Ok(())
    }

    pub fn transform_values(&self, record: &mut DataRecord) -> Result<(), ProcessingError> {
        if record.values.is_empty() {
            return Err(ProcessingError::TransformationError(
                "Cannot transform empty values".to_string(),
            ));
        }

        for value in &mut record.values {
            *value *= self.transformation_factor;
            
            if value.is_nan() || value.is_infinite() {
                return Err(ProcessingError::TransformationError(
                    "Transformation produced invalid value".to_string(),
                ));
            }
        }

        record.metadata.insert(
            "processed".to_string(),
            "true".to_string(),
        );
        record.metadata.insert(
            "transformation_factor".to_string(),
            self.transformation_factor.to_string(),
        );

        Ok(())
    }

    pub fn process_record(&self, mut record: DataRecord) -> Result<DataRecord, ProcessingError> {
        self.validate_record(&record)?;
        self.transform_values(&mut record)?;
        Ok(record)
    }

    pub fn batch_process(
        &self,
        records: Vec<DataRecord>,
    ) -> (Vec<DataRecord>, Vec<(usize, ProcessingError)>) {
        let mut processed = Vec::new();
        let mut errors = Vec::new();

        for (index, record) in records.into_iter().enumerate() {
            match self.process_record(record) {
                Ok(processed_record) => processed.push(processed_record),
                Err(err) => errors.push((index, err)),
            }
        }

        (processed, errors)
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
    fn test_validation_threshold_exceeded() {
        let processor = DataProcessor::new(2.5, 1.0);
        let record = create_test_record();
        assert!(processor.validate_record(&record).is_err());
    }

    #[test]
    fn test_transformation() {
        let processor = DataProcessor::new(100.0, 2.0);
        let mut record = create_test_record();
        assert!(processor.transform_values(&mut record).is_ok());
        assert_eq!(record.values, vec![2.0, 4.0, 6.0]);
        assert_eq!(record.metadata.get("processed"), Some(&"true".to_string()));
    }

    #[test]
    fn test_batch_processing() {
        let processor = DataProcessor::new(100.0, 2.0);
        let records = vec![create_test_record(), create_test_record()];
        let (processed, errors) = processor.batch_process(records);
        assert_eq!(processed.len(), 2);
        assert!(errors.is_empty());
    }
}