
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
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
        if record.values.is_empty() {
            return Err(ProcessingError::ValidationError(
                "Record contains no values".to_string(),
            ));
        }

        for (i, &value) in record.values.iter().enumerate() {
            if value.is_nan() || value.is_infinite() {
                return Err(ProcessingError::InvalidData(format!(
                    "Invalid value at position {}: {}",
                    i, value
                )));
            }

            if value.abs() > self.validation_threshold {
                return Err(ProcessingError::ValidationError(format!(
                    "Value {} exceeds threshold {} at position {}",
                    value, self.validation_threshold, i
                )));
            }
        }

        Ok(())
    }

    pub fn normalize_values(&self, record: &mut DataRecord) -> Result<(), ProcessingError> {
        self.validate_record(record)?;

        for value in record.values.iter_mut() {
            *value = (*value) / self.normalization_factor;
            
            if value.is_nan() || value.is_infinite() {
                return Err(ProcessingError::TransformationFailed(
                    "Normalization produced invalid result".to_string(),
                ));
            }
        }

        record.metadata.insert(
            "normalized".to_string(),
            "true".to_string(),
        );

        Ok(())
    }

    pub fn calculate_statistics(&self, records: &[DataRecord]) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        
        if records.is_empty() {
            return stats;
        }

        let value_count = records[0].values.len();
        let mut sums = vec![0.0; value_count];
        let mut squares = vec![0.0; value_count];
        let mut counts = vec![0; value_count];

        for record in records {
            for (i, &value) in record.values.iter().enumerate() {
                if i < value_count && !value.is_nan() && !value.is_infinite() {
                    sums[i] += value;
                    squares[i] += value * value;
                    counts[i] += 1;
                }
            }
        }

        for i in 0..value_count {
            if counts[i] > 0 {
                let mean = sums[i] / counts[i] as f64;
                let variance = (squares[i] / counts[i] as f64) - (mean * mean);
                
                stats.insert(format!("mean_{}", i), mean);
                stats.insert(format!("variance_{}", i), variance);
            }
        }

        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_valid_record() {
        let processor = DataProcessor::new(1000.0, 1.0);
        let record = DataRecord {
            id: 1,
            values: vec![10.5, 20.3, 30.7],
            metadata: HashMap::new(),
        };

        assert!(processor.validate_record(&record).is_ok());
    }

    #[test]
    fn test_validation_exceeds_threshold() {
        let processor = DataProcessor::new(10.0, 1.0);
        let record = DataRecord {
            id: 1,
            values: vec![5.0, 15.0, 8.0],
            metadata: HashMap::new(),
        };

        assert!(processor.validate_record(&record).is_err());
    }

    #[test]
    fn test_normalization() {
        let processor = DataProcessor::new(1000.0, 10.0);
        let mut record = DataRecord {
            id: 1,
            values: vec![100.0, 200.0, 300.0],
            metadata: HashMap::new(),
        };

        assert!(processor.normalize_values(&mut record).is_ok());
        assert_eq!(record.values, vec![10.0, 20.0, 30.0]);
        assert_eq!(record.metadata.get("normalized"), Some(&"true".to_string()));
    }

    #[test]
    fn test_statistics_calculation() {
        let processor = DataProcessor::new(1000.0, 1.0);
        let records = vec![
            DataRecord {
                id: 1,
                values: vec![1.0, 2.0],
                metadata: HashMap::new(),
            },
            DataRecord {
                id: 2,
                values: vec![3.0, 4.0],
                metadata: HashMap::new(),
            },
        ];

        let stats = processor.calculate_statistics(&records);
        
        assert_eq!(stats.get("mean_0"), Some(&2.0));
        assert_eq!(stats.get("mean_1"), Some(&3.0));
        assert_eq!(stats.get("variance_0"), Some(&1.0));
        assert_eq!(stats.get("variance_1"), Some(&1.0));
    }
}