
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DataError {
    #[error("Invalid input data")]
    InvalidInput,
    #[error("Processing timeout")]
    Timeout,
    #[error("Serialization failed: {0}")]
    Serialization(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRecord {
    pub id: u64,
    pub timestamp: i64,
    pub values: HashMap<String, f64>,
    pub tags: Vec<String>,
}

impl DataRecord {
    pub fn new(id: u64, timestamp: i64) -> Self {
        Self {
            id,
            timestamp,
            values: HashMap::new(),
            tags: Vec::new(),
        }
    }

    pub fn add_value(&mut self, key: &str, value: f64) {
        self.values.insert(key.to_string(), value);
    }

    pub fn add_tag(&mut self, tag: &str) {
        self.tags.push(tag.to_string());
    }

    pub fn validate(&self) -> Result<(), DataError> {
        if self.id == 0 {
            return Err(DataError::InvalidInput);
        }
        if self.timestamp < 0 {
            return Err(DataError::InvalidInput);
        }
        if self.values.is_empty() {
            return Err(DataError::InvalidInput);
        }
        Ok(())
    }
}

pub struct DataProcessor {
    max_records: usize,
    processing_timeout: u64,
}

impl DataProcessor {
    pub fn new(max_records: usize, processing_timeout: u64) -> Self {
        Self {
            max_records,
            processing_timeout,
        }
    }

    pub fn process_records(
        &self,
        records: Vec<DataRecord>,
    ) -> Result<Vec<DataRecord>, DataError> {
        if records.len() > self.max_records {
            return Err(DataError::InvalidInput);
        }

        let mut processed = Vec::with_capacity(records.len());
        for record in records {
            record.validate()?;
            let mut processed_record = record.clone();
            self.transform_values(&mut processed_record);
            processed.push(processed_record);
        }

        Ok(processed)
    }

    fn transform_values(&self, record: &mut DataRecord) {
        let transformed: HashMap<String, f64> = record
            .values
            .iter()
            .map(|(k, v)| (k.clone(), v * 2.0))
            .collect();
        record.values = transformed;
    }

    pub fn serialize_records(&self, records: &[DataRecord]) -> Result<String, DataError> {
        serde_json::to_string(records)
            .map_err(|e| DataError::Serialization(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_validation() {
        let mut record = DataRecord::new(1, 1234567890);
        record.add_value("temperature", 23.5);
        assert!(record.validate().is_ok());

        let invalid_record = DataRecord::new(0, 1234567890);
        assert!(invalid_record.validate().is_err());
    }

    #[test]
    fn test_data_processing() {
        let processor = DataProcessor::new(100, 5000);
        let mut record = DataRecord::new(1, 1234567890);
        record.add_value("pressure", 1013.25);

        let result = processor.process_records(vec![record]);
        assert!(result.is_ok());
        let processed = result.unwrap();
        assert_eq!(processed[0].values["pressure"], 2026.5);
    }
}