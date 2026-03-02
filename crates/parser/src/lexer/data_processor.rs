
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
    pub min_timestamp: i64,
    pub require_metadata: bool,
}

impl DataProcessor {
    pub fn new(config: ProcessingConfig) -> Self {
        DataProcessor { config }
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if record.values.len() > self.config.max_values {
            return Err(ProcessingError::ValidationFailed(
                format!("Too many values: {}", record.values.len())
            ));
        }

        if record.timestamp < self.config.min_timestamp {
            return Err(ProcessingError::ValidationFailed(
                format!("Timestamp too old: {}", record.timestamp)
            ));
        }

        if self.config.require_metadata && record.metadata.is_empty() {
            return Err(ProcessingError::ValidationFailed(
                "Metadata required but missing".to_string()
            ));
        }

        Ok(())
    }

    pub fn transform_record(&self, record: &DataRecord) -> Result<DataRecord, ProcessingError> {
        let mut transformed = record.clone();
        
        transformed.values = record.values
            .iter()
            .map(|&v| v * 2.0)
            .collect();

        transformed.metadata.insert(
            "processed_at".to_string(),
            chrono::Utc::now().to_rfc3339()
        );

        transformed.metadata.insert(
            "transformation_count".to_string(),
            "1".to_string()
        );

        Ok(transformed)
    }

    pub fn process_batch(&self, records: Vec<DataRecord>) -> Result<Vec<DataRecord>, ProcessingError> {
        let mut processed = Vec::with_capacity(records.len());

        for record in records {
            self.validate_record(&record)?;
            let transformed = self.transform_record(&record)?;
            processed.push(transformed);
        }

        Ok(processed)
    }

    pub fn calculate_statistics(&self, records: &[DataRecord]) -> HashMap<String, f64> {
        let mut stats = HashMap::new();

        let total_records = records.len() as f64;
        if total_records == 0.0 {
            return stats;
        }

        let total_values: usize = records.iter()
            .map(|r| r.values.len())
            .sum();

        let sum_all_values: f64 = records.iter()
            .flat_map(|r| &r.values)
            .sum();

        let avg_values_per_record = total_values as f64 / total_records;
        let avg_value = sum_all_values / total_values as f64;

        stats.insert("total_records".to_string(), total_records);
        stats.insert("total_values".to_string(), total_values as f64);
        stats.insert("avg_values_per_record".to_string(), avg_values_per_record);
        stats.insert("avg_value".to_string(), avg_value);

        stats
    }
}

impl Default for ProcessingConfig {
    fn default() -> Self {
        ProcessingConfig {
            max_values: 100,
            min_timestamp: 0,
            require_metadata: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_success() {
        let config = ProcessingConfig::default();
        let processor = DataProcessor::new(config);

        let record = DataRecord {
            id: 1,
            timestamp: 1000,
            values: vec![1.0, 2.0, 3.0],
            metadata: HashMap::new(),
        };

        assert!(processor.validate_record(&record).is_ok());
    }

    #[test]
    fn test_validation_too_many_values() {
        let config = ProcessingConfig {
            max_values: 2,
            ..ProcessingConfig::default()
        };
        let processor = DataProcessor::new(config);

        let record = DataRecord {
            id: 1,
            timestamp: 1000,
            values: vec![1.0, 2.0, 3.0],
            metadata: HashMap::new(),
        };

        assert!(processor.validate_record(&record).is_err());
    }

    #[test]
    fn test_transform_record() {
        let processor = DataProcessor::new(ProcessingConfig::default());
        
        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), "test".to_string());

        let record = DataRecord {
            id: 1,
            timestamp: 1000,
            values: vec![1.0, 2.0, 3.0],
            metadata,
        };

        let transformed = processor.transform_record(&record).unwrap();
        
        assert_eq!(transformed.values, vec![2.0, 4.0, 6.0]);
        assert!(transformed.metadata.contains_key("processed_at"));
        assert!(transformed.metadata.contains_key("transformation_count"));
    }

    #[test]
    fn test_calculate_statistics() {
        let processor = DataProcessor::new(ProcessingConfig::default());

        let records = vec![
            DataRecord {
                id: 1,
                timestamp: 1000,
                values: vec![1.0, 2.0],
                metadata: HashMap::new(),
            },
            DataRecord {
                id: 2,
                timestamp: 2000,
                values: vec![3.0, 4.0, 5.0],
                metadata: HashMap::new(),
            },
        ];

        let stats = processor.calculate_statistics(&records);

        assert_eq!(stats.get("total_records").unwrap(), &2.0);
        assert_eq!(stats.get("total_values").unwrap(), &5.0);
        assert_eq!(stats.get("avg_values_per_record").unwrap(), &2.5);
        assert_eq!(stats.get("avg_value").unwrap(), &3.0);
    }
}