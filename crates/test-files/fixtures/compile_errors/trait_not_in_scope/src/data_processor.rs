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

        let max_value = record
            .values
            .iter()
            .fold(f64::NEG_INFINITY, |a, &b| a.max(b));

        if max_value > self.validation_threshold {
            return Err(ProcessingError::ValidationFailed(format!(
                "Value {} exceeds threshold {}",
                max_value, self.validation_threshold
            )));
        }

        Ok(())
    }

    pub fn transform_record(&self, record: &mut DataRecord) -> Result<(), ProcessingError> {
        if record.values.iter().any(|&v| v.is_nan() || v.is_infinite()) {
            return Err(ProcessingError::TransformationError(
                "Invalid numeric values detected".to_string(),
            ));
        }

        for value in &mut record.values {
            *value *= self.transformation_factor;
        }

        record.metadata.insert(
            "processed".to_string(),
            chrono::Utc::now().to_rfc3339(),
        );

        Ok(())
    }

    pub fn process_records(
        &self,
        records: &mut [DataRecord],
    ) -> Result<Vec<DataRecord>, ProcessingError> {
        let mut processed = Vec::with_capacity(records.len());

        for record in records {
            self.validate_record(record)?;
            let mut processed_record = record.clone();
            self.transform_record(&mut processed_record)?;
            processed.push(processed_record);
        }

        Ok(processed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_success() {
        let processor = DataProcessor::new(100.0, 2.0);
        let record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: vec![10.0, 20.0, 30.0],
            metadata: HashMap::new(),
        };

        assert!(processor.validate_record(&record).is_ok());
    }

    #[test]
    fn test_validation_failure() {
        let processor = DataProcessor::new(50.0, 2.0);
        let record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: vec![10.0, 60.0, 30.0],
            metadata: HashMap::new(),
        };

        assert!(processor.validate_record(&record).is_err());
    }

    #[test]
    fn test_transformation() {
        let processor = DataProcessor::new(100.0, 3.0);
        let mut record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: vec![1.0, 2.0, 3.0],
            metadata: HashMap::new(),
        };

        processor.transform_record(&mut record).unwrap();
        assert_eq!(record.values, vec![3.0, 6.0, 9.0]);
        assert!(record.metadata.contains_key("processed"));
    }
}