
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProcessingError {
    #[error("Invalid input data")]
    InvalidInput,
    #[error("Data validation failed: {0}")]
    ValidationFailed(String),
    #[error("Transformation error")]
    TransformationError,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DataRecord {
    pub id: u64,
    pub timestamp: i64,
    pub values: HashMap<String, f64>,
    pub metadata: Option<HashMap<String, String>>,
}

impl DataRecord {
    pub fn new(id: u64, timestamp: i64) -> Self {
        Self {
            id,
            timestamp,
            values: HashMap::new(),
            metadata: None,
        }
    }

    pub fn add_value(&mut self, key: &str, value: f64) {
        self.values.insert(key.to_string(), value);
    }

    pub fn add_metadata(&mut self, key: &str, value: &str) {
        if self.metadata.is_none() {
            self.metadata = Some(HashMap::new());
        }
        if let Some(metadata) = &mut self.metadata {
            metadata.insert(key.to_string(), value.to_string());
        }
    }
}

pub struct DataProcessor {
    validation_rules: HashMap<String, ValidationRule>,
}

impl DataProcessor {
    pub fn new() -> Self {
        Self {
            validation_rules: HashMap::new(),
        }
    }

    pub fn add_validation_rule(&mut self, field: &str, rule: ValidationRule) {
        self.validation_rules.insert(field.to_string(), rule);
    }

    pub fn process_record(&self, record: &DataRecord) -> Result<ProcessedRecord, ProcessingError> {
        self.validate_record(record)?;
        let transformed = self.transform_record(record)?;
        Ok(transformed)
    }

    fn validate_record(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if record.id == 0 {
            return Err(ProcessingError::InvalidInput);
        }

        if record.timestamp <= 0 {
            return Err(ProcessingError::ValidationFailed(
                "Timestamp must be positive".to_string(),
            ));
        }

        for (field, rule) in &self.validation_rules {
            if let Some(value) = record.values.get(field) {
                if !rule.is_valid(*value) {
                    return Err(ProcessingError::ValidationFailed(format!(
                        "Field '{}' failed validation",
                        field
                    )));
                }
            }
        }

        Ok(())
    }

    fn transform_record(&self, record: &DataRecord) -> Result<ProcessedRecord, ProcessingError> {
        let mut normalized_values = HashMap::new();
        
        for (key, value) in &record.values {
            let normalized = (value * 100.0).round() / 100.0;
            normalized_values.insert(key.clone(), normalized);
        }

        let processed = ProcessedRecord {
            original_id: record.id,
            processed_timestamp: chrono::Utc::now().timestamp(),
            normalized_values,
            summary: self.generate_summary(record),
        };

        Ok(processed)
    }

    fn generate_summary(&self, record: &DataRecord) -> RecordSummary {
        let value_count = record.values.len();
        let total: f64 = record.values.values().sum();
        let average = if value_count > 0 {
            total / value_count as f64
        } else {
            0.0
        };

        RecordSummary {
            value_count,
            total,
            average,
            has_metadata: record.metadata.is_some(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ValidationRule {
    pub min_value: Option<f64>,
    pub max_value: Option<f64>,
    pub allowed_values: Option<Vec<f64>>,
}

impl ValidationRule {
    pub fn new() -> Self {
        Self {
            min_value: None,
            max_value: None,
            allowed_values: None,
        }
    }

    pub fn is_valid(&self, value: f64) -> bool {
        if let Some(min) = self.min_value {
            if value < min {
                return false;
            }
        }

        if let Some(max) = self.max_value {
            if value > max {
                return false;
            }
        }

        if let Some(allowed) = &self.allowed_values {
            if !allowed.contains(&value) {
                return false;
            }
        }

        true
    }
}

#[derive(Debug, Serialize)]
pub struct ProcessedRecord {
    pub original_id: u64,
    pub processed_timestamp: i64,
    pub normalized_values: HashMap<String, f64>,
    pub summary: RecordSummary,
}

#[derive(Debug, Serialize)]
pub struct RecordSummary {
    pub value_count: usize,
    pub total: f64,
    pub average: f64,
    pub has_metadata: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_record_creation() {
        let mut record = DataRecord::new(1, 1234567890);
        record.add_value("temperature", 25.5);
        record.add_metadata("source", "sensor_001");

        assert_eq!(record.id, 1);
        assert_eq!(record.values.get("temperature"), Some(&25.5));
        assert!(record.metadata.is_some());
    }

    #[test]
    fn test_validation_rule() {
        let mut rule = ValidationRule::new();
        rule.min_value = Some(0.0);
        rule.max_value = Some(100.0);

        assert!(rule.is_valid(50.0));
        assert!(!rule.is_valid(-10.0));
        assert!(!rule.is_valid(150.0));
    }

    #[test]
    fn test_data_processing() {
        let mut processor = DataProcessor::new();
        let mut rule = ValidationRule::new();
        rule.min_value = Some(0.0);
        rule.max_value = Some(50.0);
        processor.add_validation_rule("temperature", rule);

        let mut record = DataRecord::new(1, 1234567890);
        record.add_value("temperature", 25.5);
        record.add_value("humidity", 60.0);

        let result = processor.process_record(&record);
        assert!(result.is_ok());

        let processed = result.unwrap();
        assert_eq!(processed.original_id, 1);
        assert_eq!(processed.normalized_values.get("temperature"), Some(&25.5));
        assert_eq!(processed.summary.value_count, 2);
    }
}