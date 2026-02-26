
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
    config: ProcessingConfig,
}

#[derive(Debug, Clone)]
pub struct ProcessingConfig {
    pub max_values: usize,
    pub require_timestamp: bool,
    pub allowed_metadata_keys: Vec<String>,
}

impl DataProcessor {
    pub fn new(config: ProcessingConfig) -> Self {
        DataProcessor { config }
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if record.values.len() > self.config.max_values {
            return Err(ProcessingError::ValidationFailed(
                format!("Too many values: {} > {}", record.values.len(), self.config.max_values)
            ));
        }

        if self.config.require_timestamp && record.timestamp <= 0 {
            return Err(ProcessingError::ValidationFailed(
                "Invalid timestamp".to_string()
            ));
        }

        for key in record.metadata.keys() {
            if !self.config.allowed_metadata_keys.contains(key) {
                return Err(ProcessingError::ValidationFailed(
                    format!("Disallowed metadata key: {}", key)
                ));
            }
        }

        Ok(())
    }

    pub fn transform_record(&self, record: DataRecord) -> Result<DataRecord, ProcessingError> {
        self.validate_record(&record)?;

        let mut transformed = record.clone();
        
        transformed.values = transformed.values
            .into_iter()
            .map(|v| v * 2.0)
            .collect();

        transformed.metadata.insert(
            "processed_timestamp".to_string(),
            chrono::Utc::now().timestamp().to_string()
        );

        Ok(transformed)
    }

    pub fn batch_process(
        &self,
        records: Vec<DataRecord>
    ) -> Result<Vec<DataRecord>, Vec<ProcessingError>> {
        let mut results = Vec::new();
        let mut errors = Vec::new();

        for record in records {
            match self.transform_record(record) {
                Ok(transformed) => results.push(transformed),
                Err(e) => errors.push(e),
            }
        }

        if !errors.is_empty() {
            Err(errors)
        } else {
            Ok(results)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> ProcessingConfig {
        ProcessingConfig {
            max_values: 10,
            require_timestamp: true,
            allowed_metadata_keys: vec!["source".to_string(), "version".to_string()],
        }
    }

    #[test]
    fn test_valid_record_validation() {
        let processor = DataProcessor::new(create_test_config());
        let record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: vec![1.0, 2.0, 3.0],
            metadata: {
                let mut map = HashMap::new();
                map.insert("source".to_string(), "test".to_string());
                map
            },
        };

        assert!(processor.validate_record(&record).is_ok());
    }

    #[test]
    fn test_invalid_record_validation() {
        let processor = DataProcessor::new(create_test_config());
        let record = DataRecord {
            id: 1,
            timestamp: 0,
            values: vec![1.0; 20],
            metadata: HashMap::new(),
        };

        assert!(processor.validate_record(&record).is_err());
    }

    #[test]
    fn test_record_transformation() {
        let processor = DataProcessor::new(create_test_config());
        let record = DataRecord {
            id: 42,
            timestamp: 1000,
            values: vec![1.5, 2.5],
            metadata: HashMap::new(),
        };

        let result = processor.transform_record(record).unwrap();
        assert_eq!(result.values, vec![3.0, 5.0]);
        assert!(result.metadata.contains_key("processed_timestamp"));
    }
}