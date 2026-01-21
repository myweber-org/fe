
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
    max_records: usize,
}

impl DataProcessor {
    pub fn new(validation_threshold: f64, max_records: usize) -> Self {
        DataProcessor {
            validation_threshold,
            max_records,
        }
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if record.values.is_empty() {
            return Err(ProcessingError::ValidationFailed(
                "Empty values array".to_string(),
            ));
        }

        if record.timestamp < 0 {
            return Err(ProcessingError::ValidationFailed(
                "Negative timestamp".to_string(),
            ));
        }

        for value in &record.values {
            if value.is_nan() || value.is_infinite() {
                return Err(ProcessingError::ValidationFailed(
                    "Invalid numeric value".to_string(),
                ));
            }
        }

        Ok(())
    }

    pub fn transform_records(
        &self,
        records: Vec<DataRecord>,
    ) -> Result<Vec<DataRecord>, ProcessingError> {
        if records.len() > self.max_records {
            return Err(ProcessingError::TransformationError(format!(
                "Exceeded maximum records: {} > {}",
                records.len(),
                self.max_records
            )));
        }

        let mut transformed = Vec::with_capacity(records.len());

        for mut record in records {
            self.validate_record(&record)?;

            let avg_value: f64 = record.values.iter().sum::<f64>() / record.values.len() as f64;
            
            if avg_value > self.validation_threshold {
                record.metadata.insert(
                    "processing_status".to_string(),
                    "threshold_exceeded".to_string(),
                );
            } else {
                record.metadata.insert(
                    "processing_status".to_string(),
                    "within_threshold".to_string(),
                );
            }

            record.metadata.insert(
                "processed_timestamp".to_string(),
                chrono::Utc::now().timestamp().to_string(),
            );

            transformed.push(record);
        }

        Ok(transformed)
    }

    pub fn filter_records(
        &self,
        records: Vec<DataRecord>,
        predicate: impl Fn(&DataRecord) -> bool,
    ) -> Vec<DataRecord> {
        records.into_iter().filter(predicate).collect()
    }

    pub fn calculate_statistics(&self, records: &[DataRecord]) -> HashMap<String, f64> {
        let mut stats = HashMap::new();

        if records.is_empty() {
            return stats;
        }

        let total_values: usize = records.iter().map(|r| r.values.len()).sum();
        let all_values: Vec<f64> = records
            .iter()
            .flat_map(|r| r.values.clone())
            .collect();

        stats.insert("total_records".to_string(), records.len() as f64);
        stats.insert("total_values".to_string(), total_values as f64);

        if !all_values.is_empty() {
            let sum: f64 = all_values.iter().sum();
            let count = all_values.len() as f64;
            let mean = sum / count;

            let variance: f64 = all_values
                .iter()
                .map(|&value| {
                    let diff = mean - value;
                    diff * diff
                })
                .sum::<f64>()
                / count;

            stats.insert("mean".to_string(), mean);
            stats.insert("variance".to_string(), variance);
            stats.insert("min".to_string(), all_values.iter().fold(f64::INFINITY, |a, &b| a.min(b)));
            stats.insert("max".to_string(), all_values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b)));
        }

        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_record() {
        let processor = DataProcessor::new(100.0, 1000);
        let mut record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: vec![10.0, 20.0, 30.0],
            metadata: HashMap::new(),
        };

        assert!(processor.validate_record(&record).is_ok());

        record.values = vec![];
        assert!(processor.validate_record(&record).is_err());

        record.values = vec![10.0, f64::NAN, 30.0];
        assert!(processor.validate_record(&record).is_err());
    }

    #[test]
    fn test_transform_records() {
        let processor = DataProcessor::new(50.0, 10);
        let records = vec![
            DataRecord {
                id: 1,
                timestamp: 1234567890,
                values: vec![10.0, 20.0, 30.0],
                metadata: HashMap::new(),
            },
            DataRecord {
                id: 2,
                timestamp: 1234567891,
                values: vec![60.0, 70.0, 80.0],
                metadata: HashMap::new(),
            },
        ];

        let result = processor.transform_records(records);
        assert!(result.is_ok());

        let transformed = result.unwrap();
        assert_eq!(transformed.len(), 2);
        assert!(transformed[0]
            .metadata
            .contains_key("processing_status"));
        assert!(transformed[0]
            .metadata
            .contains_key("processed_timestamp"));
    }

    #[test]
    fn test_calculate_statistics() {
        let processor = DataProcessor::new(100.0, 1000);
        let records = vec![
            DataRecord {
                id: 1,
                timestamp: 1234567890,
                values: vec![10.0, 20.0, 30.0],
                metadata: HashMap::new(),
            },
            DataRecord {
                id: 2,
                timestamp: 1234567891,
                values: vec![40.0, 50.0, 60.0],
                metadata: HashMap::new(),
            },
        ];

        let stats = processor.calculate_statistics(&records);
        assert_eq!(stats["total_records"], 2.0);
        assert_eq!(stats["total_values"], 6.0);
        assert_eq!(stats["mean"], 35.0);
        assert_eq!(stats["min"], 10.0);
        assert_eq!(stats["max"], 60.0);
    }
}