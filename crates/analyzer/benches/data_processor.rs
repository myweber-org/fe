
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

#[derive(Debug, Error)]
pub enum ProcessingError {
    #[error("Invalid data format")]
    InvalidFormat,
    #[error("Data validation failed: {0}")]
    ValidationFailed(String),
    #[error("Transformation error: {0}")]
    TransformationError(String),
}

pub struct DataProcessor {
    validation_rules: Vec<Box<dyn Fn(&DataRecord) -> Result<(), ProcessingError>>>,
    transformation_pipeline: Vec<Box<dyn Fn(DataRecord) -> Result<DataRecord, ProcessingError>>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            validation_rules: Vec::new(),
            transformation_pipeline: Vec::new(),
        }
    }

    pub fn add_validation_rule<F>(&mut self, rule: F)
    where
        F: Fn(&DataRecord) -> Result<(), ProcessingError> + 'static,
    {
        self.validation_rules.push(Box::new(rule));
    }

    pub fn add_transformation<F>(&mut self, transform: F)
    where
        F: Fn(DataRecord) -> Result<DataRecord, ProcessingError> + 'static,
    {
        self.transformation_pipeline.push(Box::new(transform));
    }

    pub fn process(&self, mut record: DataRecord) -> Result<DataRecord, ProcessingError> {
        for rule in &self.validation_rules {
            rule(&record)?;
        }

        for transform in &self.transformation_pipeline {
            record = transform(record)?;
        }

        Ok(record)
    }

    pub fn batch_process(
        &self,
        records: Vec<DataRecord>,
    ) -> Result<Vec<DataRecord>, ProcessingError> {
        let mut results = Vec::with_capacity(records.len());

        for record in records {
            match self.process(record) {
                Ok(processed) => results.push(processed),
                Err(e) => return Err(e),
            }
        }

        Ok(results)
    }
}

fn validate_timestamp(record: &DataRecord) -> Result<(), ProcessingError> {
    if record.timestamp <= 0 {
        Err(ProcessingError::ValidationFailed(
            "Timestamp must be positive".to_string(),
        ))
    } else {
        Ok(())
    }
}

fn validate_values(record: &DataRecord) -> Result<(), ProcessingError> {
    if record.values.is_empty() {
        Err(ProcessingError::ValidationFailed(
            "Values cannot be empty".to_string(),
        ))
    } else if record.values.iter().any(|&v| v.is_nan() || v.is_infinite()) {
        Err(ProcessingError::ValidationFailed(
            "Values contain invalid numbers".to_string(),
        ))
    } else {
        Ok(())
    }
}

fn normalize_values(record: DataRecord) -> Result<DataRecord, ProcessingError> {
    if record.values.is_empty() {
        return Ok(record);
    }

    let sum: f64 = record.values.iter().sum();
    if sum == 0.0 {
        return Err(ProcessingError::TransformationError(
            "Cannot normalize zero-sum vector".to_string(),
        ));
    }

    let normalized_values: Vec<f64> = record.values.iter().map(|&v| v / sum).collect();

    Ok(DataRecord {
        values: normalized_values,
        ..record
    })
}

pub fn create_default_processor() -> DataProcessor {
    let mut processor = DataProcessor::new();
    processor.add_validation_rule(validate_timestamp);
    processor.add_validation_rule(validate_values);
    processor.add_transformation(normalize_values);
    processor
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
        let record = create_test_record();
        let processor = create_default_processor();
        assert!(processor.process(record).is_ok());
    }

    #[test]
    fn test_validation_failure() {
        let mut record = create_test_record();
        record.timestamp = -1;
        let processor = create_default_processor();
        assert!(processor.process(record).is_err());
    }

    #[test]
    fn test_normalization() {
        let record = create_test_record();
        let processor = create_default_processor();
        let result = processor.process(record).unwrap();
        let sum: f64 = result.values.iter().sum();
        assert!((sum - 1.0).abs() < 1e-10);
    }
}