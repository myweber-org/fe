
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRecord {
    pub id: u64,
    pub timestamp: i64,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug)]
pub enum ProcessingError {
    InvalidData(String),
    TransformationFailed(String),
    ValidationError(String),
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessingError::InvalidData(msg) => write!(f, "Invalid data: {}", msg),
            ProcessingError::TransformationFailed(msg) => write!(f, "Transformation failed: {}", msg),
            ProcessingError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl Error for ProcessingError {}

pub struct DataProcessor {
    validation_threshold: f64,
    normalization_factor: f64,
}

impl DataProcessor {
    pub fn new(validation_threshold: f64, normalization_factor: f64) -> Self {
        DataProcessor {
            validation_threshold,
            normalization_factor,
        }
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if record.id == 0 {
            return Err(ProcessingError::ValidationError("ID cannot be zero".to_string()));
        }

        if record.timestamp < 0 {
            return Err(ProcessingError::ValidationError("Timestamp cannot be negative".to_string()));
        }

        if record.values.is_empty() {
            return Err(ProcessingError::ValidationError("Values cannot be empty".to_string()));
        }

        for value in &record.values {
            if value.is_nan() || value.is_infinite() {
                return Err(ProcessingError::ValidationError("Invalid numeric value detected".to_string()));
            }
        }

        Ok(())
    }

    pub fn transform_values(&self, record: &mut DataRecord) -> Result<(), ProcessingError> {
        if record.values.iter().any(|&v| v.abs() > self.validation_threshold) {
            return Err(ProcessingError::TransformationFailed(
                format!("Value exceeds threshold {}", self.validation_threshold)
            ));
        }

        for value in &mut record.values {
            *value = (*value * self.normalization_factor).round() / self.normalization_factor;
        }

        record.metadata.insert(
            "processed".to_string(),
            "true".to_string()
        );

        Ok(())
    }

    pub fn calculate_statistics(&self, record: &DataRecord) -> HashMap<String, f64> {
        let mut stats = HashMap::new();

        if record.values.is_empty() {
            return stats;
        }

        let sum: f64 = record.values.iter().sum();
        let count = record.values.len() as f64;
        let mean = sum / count;

        let variance: f64 = record.values.iter()
            .map(|&value| (value - mean).powi(2))
            .sum::<f64>() / count;

        stats.insert("mean".to_string(), mean);
        stats.insert("variance".to_string(), variance);
        stats.insert("count".to_string(), count);
        stats.insert("sum".to_string(), sum);

        if let Some(min) = record.values.iter().min_by(|a, b| a.partial_cmp(b).unwrap()) {
            stats.insert("min".to_string(), *min);
        }

        if let Some(max) = record.values.iter().max_by(|a, b| a.partial_cmp(b).unwrap()) {
            stats.insert("max".to_string(), *max);
        }

        stats
    }

    pub fn process_record(&self, mut record: DataRecord) -> Result<(DataRecord, HashMap<String, f64>), ProcessingError> {
        self.validate_record(&record)?;
        self.transform_values(&mut record)?;
        let stats = self.calculate_statistics(&record);
        
        Ok((record, stats))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_success() {
        let processor = DataProcessor::new(1000.0, 100.0);
        let record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: vec![1.0, 2.0, 3.0],
            metadata: HashMap::new(),
        };

        assert!(processor.validate_record(&record).is_ok());
    }

    #[test]
    fn test_validation_failure() {
        let processor = DataProcessor::new(1000.0, 100.0);
        let record = DataRecord {
            id: 0,
            timestamp: -1,
            values: vec![],
            metadata: HashMap::new(),
        };

        assert!(processor.validate_record(&record).is_err());
    }

    #[test]
    fn test_transform_values() {
        let processor = DataProcessor::new(1000.0, 10.0);
        let mut record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: vec![1.234, 2.567, 3.891],
            metadata: HashMap::new(),
        };

        assert!(processor.transform_values(&mut record).is_ok());
        assert_eq!(record.values, vec![1.2, 2.6, 3.9]);
        assert_eq!(record.metadata.get("processed"), Some(&"true".to_string()));
    }

    #[test]
    fn test_calculate_statistics() {
        let processor = DataProcessor::new(1000.0, 100.0);
        let record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: vec![1.0, 2.0, 3.0, 4.0, 5.0],
            metadata: HashMap::new(),
        };

        let stats = processor.calculate_statistics(&record);
        
        assert_eq!(stats.get("mean"), Some(&3.0));
        assert_eq!(stats.get("sum"), Some(&15.0));
        assert_eq!(stats.get("count"), Some(&5.0));
        assert_eq!(stats.get("min"), Some(&1.0));
        assert_eq!(stats.get("max"), Some(&5.0));
    }
}