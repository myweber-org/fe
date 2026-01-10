
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataPoint {
    timestamp: i64,
    value: f64,
    category: String,
}

#[derive(Debug)]
pub enum ProcessingError {
    InvalidTimestamp,
    InvalidValue,
    EmptyCategory,
    TransformationFailed,
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessingError::InvalidTimestamp => write!(f, "Timestamp must be positive"),
            ProcessingError::InvalidValue => write!(f, "Value must be finite"),
            ProcessingError::EmptyCategory => write!(f, "Category cannot be empty"),
            ProcessingError::TransformationFailed => write!(f, "Data transformation failed"),
        }
    }
}

impl Error for ProcessingError {}

impl DataPoint {
    pub fn new(timestamp: i64, value: f64, category: String) -> Result<Self, ProcessingError> {
        if timestamp <= 0 {
            return Err(ProcessingError::InvalidTimestamp);
        }
        
        if !value.is_finite() {
            return Err(ProcessingError::InvalidValue);
        }
        
        if category.trim().is_empty() {
            return Err(ProcessingError::EmptyCategory);
        }
        
        Ok(Self {
            timestamp,
            value,
            category,
        })
    }
    
    pub fn transform(&self, multiplier: f64) -> Result<Self, ProcessingError> {
        if !multiplier.is_finite() || multiplier == 0.0 {
            return Err(ProcessingError::TransformationFailed);
        }
        
        let transformed_value = self.value * multiplier;
        
        DataPoint::new(
            self.timestamp,
            transformed_value,
            self.category.clone()
        )
    }
    
    pub fn normalize(&self, max_value: f64) -> Result<Self, ProcessingError> {
        if max_value <= 0.0 || !max_value.is_finite() {
            return Err(ProcessingError::TransformationFailed);
        }
        
        let normalized_value = self.value / max_value;
        
        DataPoint::new(
            self.timestamp,
            normalized_value,
            self.category.clone()
        )
    }
}

pub fn process_dataset(
    data: Vec<DataPoint>,
    transformation_fn: fn(&DataPoint) -> Result<DataPoint, ProcessingError>
) -> Result<Vec<DataPoint>, ProcessingError> {
    let mut results = Vec::with_capacity(data.len());
    
    for point in data {
        let transformed = transformation_fn(&point)?;
        results.push(transformed);
    }
    
    Ok(results)
}

pub fn calculate_statistics(data: &[DataPoint]) -> (f64, f64, f64) {
    if data.is_empty() {
        return (0.0, 0.0, 0.0);
    }
    
    let sum: f64 = data.iter().map(|p| p.value).sum();
    let count = data.len() as f64;
    let mean = sum / count;
    
    let variance: f64 = data.iter()
        .map(|p| (p.value - mean).powi(2))
        .sum::<f64>() / count;
    
    let std_dev = variance.sqrt();
    
    (mean, variance, std_dev)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_valid_datapoint() {
        let point = DataPoint::new(1234567890, 42.5, "temperature".to_string());
        assert!(point.is_ok());
    }
    
    #[test]
    fn test_invalid_timestamp() {
        let point = DataPoint::new(-1, 42.5, "temperature".to_string());
        assert!(matches!(point, Err(ProcessingError::InvalidTimestamp)));
    }
    
    #[test]
    fn test_transform() {
        let point = DataPoint::new(1234567890, 10.0, "pressure".to_string()).unwrap();
        let transformed = point.transform(2.5).unwrap();
        assert_eq!(transformed.value, 25.0);
    }
    
    #[test]
    fn test_normalize() {
        let point = DataPoint::new(1234567890, 75.0, "humidity".to_string()).unwrap();
        let normalized = point.normalize(100.0).unwrap();
        assert_eq!(normalized.value, 0.75);
    }
    
    #[test]
    fn test_statistics() {
        let points = vec![
            DataPoint::new(1, 10.0, "test".to_string()).unwrap(),
            DataPoint::new(2, 20.0, "test".to_string()).unwrap(),
            DataPoint::new(3, 30.0, "test".to_string()).unwrap(),
        ];
        
        let (mean, variance, std_dev) = calculate_statistics(&points);
        assert_eq!(mean, 20.0);
        assert_eq!(variance, 66.66666666666667);
        assert_eq!(std_dev, 8.16496580927726);
    }
}
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

        for value in &record.values {
            if value.is_nan() || value.is_infinite() {
                return Err(ProcessingError::ValidationFailed(
                    "Invalid numeric value".to_string(),
                ));
            }

            if value.abs() > self.validation_threshold {
                return Err(ProcessingError::ValidationFailed(format!(
                    "Value {} exceeds threshold {}",
                    value, self.validation_threshold
                )));
            }
        }

        Ok(())
    }

    pub fn transform_values(&self, record: &mut DataRecord) -> Result<(), ProcessingError> {
        if record.values.iter().any(|v| v.is_nan()) {
            return Err(ProcessingError::TransformationError(
                "Cannot transform NaN values".to_string(),
            ));
        }

        for value in &mut record.values {
            *value *= self.transformation_factor;
            *value = value.round();
        }

        record.metadata.insert(
            "processed".to_string(),
            chrono::Utc::now().to_rfc3339(),
        );

        Ok(())
    }

    pub fn process_record(&self, mut record: DataRecord) -> Result<DataRecord, ProcessingError> {
        self.validate_record(&record)?;
        self.transform_values(&mut record)?;
        Ok(record)
    }
}

pub fn calculate_statistics(records: &[DataRecord]) -> HashMap<String, f64> {
    let mut stats = HashMap::new();

    if records.is_empty() {
        return stats;
    }

    let total_values: usize = records.iter().map(|r| r.values.len()).sum();
    let all_values: Vec<f64> = records
        .iter()
        .flat_map(|r| r.values.clone())
        .collect();

    if !all_values.is_empty() {
        let sum: f64 = all_values.iter().sum();
        let count = all_values.len() as f64;
        let mean = sum / count;

        let variance: f64 = all_values
            .iter()
            .map(|v| (v - mean).powi(2))
            .sum::<f64>()
            / count;

        stats.insert("mean".to_string(), mean);
        stats.insert("variance".to_string(), variance);
        stats.insert("total_records".to_string(), records.len() as f64);
        stats.insert("total_values".to_string(), total_values as f64);
        stats.insert("min".to_string(), *all_values.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap());
        stats.insert("max".to_string(), *all_values.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap());
    }

    stats
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_success() {
        let processor = DataProcessor::new(1000.0, 2.0);
        let record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: vec![10.5, 20.3, 30.7],
            metadata: HashMap::new(),
        };

        assert!(processor.validate_record(&record).is_ok());
    }

    #[test]
    fn test_validation_threshold_exceeded() {
        let processor = DataProcessor::new(10.0, 2.0);
        let record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: vec![5.0, 15.0, 8.0],
            metadata: HashMap::new(),
        };

        assert!(processor.validate_record(&record).is_err());
    }

    #[test]
    fn test_transform_values() {
        let processor = DataProcessor::new(1000.0, 2.5);
        let mut record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: vec![2.0, 4.0, 6.0],
            metadata: HashMap::new(),
        };

        assert!(processor.transform_values(&mut record).is_ok());
        assert_eq!(record.values, vec![5.0, 10.0, 15.0]);
        assert!(record.metadata.contains_key("processed"));
    }

    #[test]
    fn test_calculate_statistics() {
        let records = vec![
            DataRecord {
                id: 1,
                timestamp: 1234567890,
                values: vec![1.0, 2.0, 3.0],
                metadata: HashMap::new(),
            },
            DataRecord {
                id: 2,
                timestamp: 1234567891,
                values: vec![4.0, 5.0, 6.0],
                metadata: HashMap::new(),
            },
        ];

        let stats = calculate_statistics(&records);
        
        assert_eq!(stats["mean"], 3.5);
        assert_eq!(stats["total_records"], 2.0);
        assert_eq!(stats["total_values"], 6.0);
        assert_eq!(stats["min"], 1.0);
        assert_eq!(stats["max"], 6.0);
    }
}