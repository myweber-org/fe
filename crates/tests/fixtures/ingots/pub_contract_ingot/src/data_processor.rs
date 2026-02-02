
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DataRecord {
    pub id: u64,
    pub timestamp: i64,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Error)]
pub enum DataError {
    #[error("Invalid timestamp: {0}")]
    InvalidTimestamp(i64),
    #[error("Empty values array")]
    EmptyValues,
    #[error("NaN value detected at index {0}")]
    NaNValue(usize),
    #[error("Duplicate record ID: {0}")]
    DuplicateId(u64),
}

pub struct DataProcessor {
    processed_ids: std::collections::HashSet<u64>,
    statistics: ProcessingStats,
}

#[derive(Debug, Default)]
pub struct ProcessingStats {
    pub total_records: usize,
    pub valid_records: usize,
    pub invalid_records: usize,
    pub average_values_per_record: f64,
}

impl DataProcessor {
    pub fn new() -> Self {
        Self {
            processed_ids: std::collections::HashSet::new(),
            statistics: ProcessingStats::default(),
        }
    }

    pub fn process_record(&mut self, record: &DataRecord) -> Result<DataRecord, DataError> {
        self.statistics.total_records += 1;

        if self.processed_ids.contains(&record.id) {
            self.statistics.invalid_records += 1;
            return Err(DataError::DuplicateId(record.id));
        }

        self.validate_timestamp(record.timestamp)?;
        self.validate_values(&record.values)?;

        let processed_record = self.transform_record(record);
        self.processed_ids.insert(record.id);
        self.statistics.valid_records += 1;

        self.update_statistics(&record);
        Ok(processed_record)
    }

    fn validate_timestamp(&self, timestamp: i64) -> Result<(), DataError> {
        if timestamp < 0 {
            Err(DataError::InvalidTimestamp(timestamp))
        } else {
            Ok(())
        }
    }

    fn validate_values(&self, values: &[f64]) -> Result<(), DataError> {
        if values.is_empty() {
            return Err(DataError::EmptyValues);
        }

        for (index, &value) in values.iter().enumerate() {
            if value.is_nan() {
                return Err(DataError::NaNValue(index));
            }
        }

        Ok(())
    }

    fn transform_record(&self, record: &DataRecord) -> DataRecord {
        let normalized_values: Vec<f64> = record
            .values
            .iter()
            .map(|&v| v.clamp(0.0, 1.0))
            .collect();

        let mut enhanced_metadata = record.metadata.clone();
        enhanced_metadata.insert(
            "processed_timestamp".to_string(),
            chrono::Utc::now().timestamp().to_string(),
        );
        enhanced_metadata.insert("values_count".to_string(), record.values.len().to_string());

        DataRecord {
            id: record.id,
            timestamp: record.timestamp,
            values: normalized_values,
            metadata: enhanced_metadata,
        }
    }

    fn update_statistics(&mut self, record: &DataRecord) {
        let total_values = self.statistics.valid_records * record.values.len();
        self.statistics.average_values_per_record = if self.statistics.valid_records > 0 {
            total_values as f64 / self.statistics.valid_records as f64
        } else {
            0.0
        };
    }

    pub fn get_statistics(&self) -> &ProcessingStats {
        &self.statistics
    }

    pub fn reset(&mut self) {
        self.processed_ids.clear();
        self.statistics = ProcessingStats::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_record_processing() {
        let mut processor = DataProcessor::new();
        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), "test".to_string());

        let record = DataRecord {
            id: 1,
            timestamp: 1625097600,
            values: vec![0.5, 0.8, 0.2],
            metadata,
        };

        let result = processor.process_record(&record);
        assert!(result.is_ok());
        assert_eq!(processor.get_statistics().valid_records, 1);
    }

    #[test]
    fn test_invalid_timestamp() {
        let mut processor = DataProcessor::new();
        let record = DataRecord {
            id: 1,
            timestamp: -1,
            values: vec![0.5],
            metadata: HashMap::new(),
        };

        let result = processor.process_record(&record);
        assert!(matches!(result, Err(DataError::InvalidTimestamp(-1))));
    }

    #[test]
    fn test_duplicate_id() {
        let mut processor = DataProcessor::new();
        let record1 = DataRecord {
            id: 1,
            timestamp: 1625097600,
            values: vec![0.5],
            metadata: HashMap::new(),
        };

        let record2 = DataRecord {
            id: 1,
            timestamp: 1625097601,
            values: vec![0.6],
            metadata: HashMap::new(),
        };

        let _ = processor.process_record(&record1);
        let result = processor.process_record(&record2);
        assert!(matches!(result, Err(DataError::DuplicateId(1))));
    }
}
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Debug, Serialize, Deserialize)]
pub struct DataRecord {
    id: u32,
    value: f64,
    category: String,
}

pub fn validate_record(record: &DataRecord) -> Result<(), Box<dyn Error>> {
    if record.id == 0 {
        return Err("ID cannot be zero".into());
    }
    
    if record.value < 0.0 {
        return Err("Value must be non-negative".into());
    }
    
    if record.category.is_empty() {
        return Err("Category cannot be empty".into());
    }
    
    Ok(())
}

pub fn transform_value(record: &mut DataRecord, multiplier: f64) {
    record.value *= multiplier;
}

pub fn filter_records(records: Vec<DataRecord>, min_value: f64) -> Vec<DataRecord> {
    records.into_iter()
        .filter(|r| r.value >= min_value)
        .collect()
}

pub fn calculate_average(records: &[DataRecord]) -> Option<f64> {
    if records.is_empty() {
        return None;
    }
    
    let sum: f64 = records.iter().map(|r| r.value).sum();
    Some(sum / records.len() as f64)
}
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct ValidationError {
    message: String,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Validation error: {}", self.message)
    }
}

impl Error for ValidationError {}

pub struct DataProcessor {
    threshold: f64,
}

impl DataProcessor {
    pub fn new(threshold: f64) -> Result<Self, ValidationError> {
        if threshold < 0.0 || threshold > 1.0 {
            return Err(ValidationError {
                message: format!("Threshold {} must be between 0.0 and 1.0", threshold),
            });
        }
        Ok(DataProcessor { threshold })
    }

    pub fn process_data(&self, input: Vec<f64>) -> Result<Vec<f64>, ValidationError> {
        if input.is_empty() {
            return Err(ValidationError {
                message: "Input data cannot be empty".to_string(),
            });
        }

        let filtered: Vec<f64> = input
            .into_iter()
            .filter(|&value| value >= self.threshold)
            .collect();

        if filtered.is_empty() {
            return Err(ValidationError {
                message: format!("No values above threshold {}", self.threshold),
            });
        }

        let max_value = filtered.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let normalized: Vec<f64> = filtered
            .iter()
            .map(|&value| value / max_value)
            .collect();

        Ok(normalized)
    }

    pub fn calculate_statistics(&self, data: &[f64]) -> (f64, f64, f64) {
        let count = data.len() as f64;
        let sum: f64 = data.iter().sum();
        let mean = sum / count;

        let variance: f64 = data
            .iter()
            .map(|value| {
                let diff = mean - value;
                diff * diff
            })
            .sum::<f64>()
            / count;

        let std_dev = variance.sqrt();

        (mean, variance, std_dev)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_processor_creation() {
        let processor = DataProcessor::new(0.5);
        assert!(processor.is_ok());
    }

    #[test]
    fn test_invalid_processor_creation() {
        let processor = DataProcessor::new(1.5);
        assert!(processor.is_err());
    }

    #[test]
    fn test_data_processing() {
        let processor = DataProcessor::new(0.3).unwrap();
        let input = vec![0.1, 0.4, 0.5, 0.2, 0.8];
        let result = processor.process_data(input);
        assert!(result.is_ok());
        let processed = result.unwrap();
        assert_eq!(processed.len(), 3);
    }

    #[test]
    fn test_statistics_calculation() {
        let processor = DataProcessor::new(0.0).unwrap();
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let (mean, variance, std_dev) = processor.calculate_statistics(&data);
        assert_eq!(mean, 3.0);
        assert_eq!(variance, 2.0);
        assert_eq!(std_dev, 2.0_f64.sqrt());
    }
}