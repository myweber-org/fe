
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
    validation_rules: Vec<ValidationRule>,
    transformation_pipeline: Vec<Transformation>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            validation_rules: Vec::new(),
            transformation_pipeline: Vec::new(),
        }
    }

    pub fn add_validation_rule(&mut self, rule: ValidationRule) {
        self.validation_rules.push(rule);
    }

    pub fn add_transformation(&mut self, transformation: Transformation) {
        self.transformation_pipeline.push(transformation);
    }

    pub fn process(&self, record: &mut DataRecord) -> Result<(), ProcessingError> {
        for rule in &self.validation_rules {
            rule.validate(record)?;
        }

        for transformation in &self.transformation_pipeline {
            transformation.apply(record)?;
        }

        Ok(())
    }

    pub fn batch_process(&self, records: &mut [DataRecord]) -> Vec<Result<(), ProcessingError>> {
        records
            .iter_mut()
            .map(|record| self.process(record))
            .collect()
    }
}

pub trait ValidationRule {
    fn validate(&self, record: &DataRecord) -> Result<(), ProcessingError>;
}

pub trait Transformation {
    fn apply(&self, record: &mut DataRecord) -> Result<(), ProcessingError>;
}

pub struct TimestampValidator {
    pub min_timestamp: i64,
    pub max_timestamp: i64,
}

impl ValidationRule for TimestampValidator {
    fn validate(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if record.timestamp < self.min_timestamp || record.timestamp > self.max_timestamp {
            Err(ProcessingError::ValidationFailed(format!(
                "Timestamp {} out of range [{}, {}]",
                record.timestamp, self.min_timestamp, self.max_timestamp
            )))
        } else {
            Ok(())
        }
    }
}

pub struct ValueNormalizer {
    pub scale_factor: f64,
}

impl Transformation for ValueNormalizer {
    fn apply(&self, record: &mut DataRecord) -> Result<(), ProcessingError> {
        if self.scale_factor == 0.0 {
            return Err(ProcessingError::TransformationError(
                "Scale factor cannot be zero".to_string(),
            ));
        }

        for value in &mut record.values {
            *value /= self.scale_factor;
        }

        Ok(())
    }
}

pub struct MetadataEnricher {
    pub additional_data: HashMap<String, String>,
}

impl Transformation for MetadataEnricher {
    fn apply(&self, record: &mut DataRecord) -> Result<(), ProcessingError> {
        for (key, value) in &self.additional_data {
            record.metadata.insert(key.clone(), value.clone());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_processing_pipeline() {
        let mut processor = DataProcessor::new();
        
        processor.add_validation_rule(TimestampValidator {
            min_timestamp: 0,
            max_timestamp: 1000,
        });

        processor.add_transformation(ValueNormalizer {
            scale_factor: 10.0,
        });

        let mut test_data = HashMap::new();
        test_data.insert("source".to_string(), "test".to_string());
        
        processor.add_transformation(MetadataEnricher {
            additional_data: test_data,
        });

        let mut record = DataRecord {
            id: 1,
            timestamp: 500,
            values: vec![100.0, 200.0, 300.0],
            metadata: HashMap::new(),
        };

        let result = processor.process(&mut record);
        assert!(result.is_ok());
        assert_eq!(record.values, vec![10.0, 20.0, 30.0]);
        assert_eq!(record.metadata.get("source"), Some(&"test".to_string()));
    }

    #[test]
    fn test_validation_failure() {
        let mut processor = DataProcessor::new();
        
        processor.add_validation_rule(TimestampValidator {
            min_timestamp: 0,
            max_timestamp: 1000,
        });

        let mut record = DataRecord {
            id: 1,
            timestamp: 1500,
            values: vec![100.0],
            metadata: HashMap::new(),
        };

        let result = processor.process(&mut record);
        assert!(result.is_err());
    }
}