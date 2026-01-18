
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProcessingError {
    #[error("Invalid data format")]
    InvalidFormat,
    #[error("Missing required field: {0}")]
    MissingField(String),
    #[error("Validation failed: {0}")]
    ValidationFailed(String),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DataRecord {
    pub id: u64,
    pub timestamp: i64,
    pub values: HashMap<String, f64>,
    pub tags: Vec<String>,
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
            transformation.apply(record);
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
    fn apply(&self, record: &mut DataRecord);
}

pub struct RequiredFieldRule {
    field_name: String,
}

impl RequiredFieldRule {
    pub fn new(field_name: &str) -> Self {
        RequiredFieldRule {
            field_name: field_name.to_string(),
        }
    }
}

impl ValidationRule for RequiredFieldRule {
    fn validate(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if !record.values.contains_key(&self.field_name) {
            return Err(ProcessingError::MissingField(self.field_name.clone()));
        }
        Ok(())
    }
}

pub struct NormalizeTransformation {
    factor: f64,
}

impl NormalizeTransformation {
    pub fn new(factor: f64) -> Self {
        NormalizeTransformation { factor }
    }
}

impl Transformation for NormalizeTransformation {
    fn apply(&self, record: &mut DataRecord) {
        for value in record.values.values_mut() {
            *value /= self.factor;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_required_field_validation() {
        let mut processor = DataProcessor::new();
        processor.add_validation_rule(RequiredFieldRule::new("temperature"));

        let mut record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: HashMap::new(),
            tags: vec![],
        };

        let result = processor.process(&mut record);
        assert!(result.is_err());

        record.values.insert("temperature".to_string(), 25.5);
        let result = processor.process(&mut record);
        assert!(result.is_ok());
    }

    #[test]
    fn test_normalize_transformation() {
        let mut processor = DataProcessor::new();
        processor.add_transformation(NormalizeTransformation::new(10.0));

        let mut record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: HashMap::from([
                ("value1".to_string(), 100.0),
                ("value2".to_string(), 50.0),
            ]),
            tags: vec![],
        };

        processor.process(&mut record).unwrap();
        assert_eq!(record.values.get("value1"), Some(&10.0));
        assert_eq!(record.values.get("value2"), Some(&5.0));
    }
}