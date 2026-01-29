
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
pub enum ValidationError {
    InvalidId,
    EmptyValues,
    ValueOutOfRange(f64),
    MissingMetadata(String),
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::InvalidId => write!(f, "Invalid record ID"),
            ValidationError::EmptyValues => write!(f, "Record values cannot be empty"),
            ValidationError::ValueOutOfRange(val) => write!(f, "Value {} is out of valid range", val),
            ValidationError::MissingMetadata(key) => write!(f, "Missing required metadata: {}", key),
        }
    }
}

impl Error for ValidationError {}

pub struct DataProcessor {
    min_value: f64,
    max_value: f64,
    required_metadata: Vec<String>,
}

impl DataProcessor {
    pub fn new(min_value: f64, max_value: f64, required_metadata: Vec<String>) -> Self {
        DataProcessor {
            min_value,
            max_value,
            required_metadata,
        }
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), ValidationError> {
        if record.id == 0 {
            return Err(ValidationError::InvalidId);
        }

        if record.values.is_empty() {
            return Err(ValidationError::EmptyValues);
        }

        for &value in &record.values {
            if value < self.min_value || value > self.max_value {
                return Err(ValidationError::ValueOutOfRange(value));
            }
        }

        for required_key in &self.required_metadata {
            if !record.metadata.contains_key(required_key) {
                return Err(ValidationError::MissingMetadata(required_key.clone()));
            }
        }

        Ok(())
    }

    pub fn normalize_values(&self, record: &mut DataRecord) {
        let min_val = record.values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max_val = record.values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        
        if max_val > min_val {
            for value in &mut record.values {
                *value = (*value - min_val) / (max_val - min_val);
            }
        }
    }

    pub fn process_records(&self, records: &mut [DataRecord]) -> Vec<Result<(), ValidationError>> {
        let mut results = Vec::new();
        
        for record in records {
            match self.validate_record(record) {
                Ok(()) => {
                    self.normalize_values(record);
                    results.push(Ok(()));
                }
                Err(err) => {
                    results.push(Err(err));
                }
            }
        }
        
        results
    }

    pub fn calculate_statistics(&self, records: &[DataRecord]) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        
        if records.is_empty() {
            return stats;
        }

        let value_count = records[0].values.len();
        let mut sums = vec![0.0; value_count];
        let mut squares = vec![0.0; value_count];
        
        for record in records {
            for (i, &value) in record.values.iter().enumerate() {
                sums[i] += value;
                squares[i] += value * value;
            }
        }

        let record_count = records.len() as f64;
        
        for i in 0..value_count {
            let mean = sums[i] / record_count;
            let variance = (squares[i] / record_count) - (mean * mean);
            
            stats.insert(format!("mean_{}", i), mean);
            stats.insert(format!("variance_{}", i), variance);
            stats.insert(format!("std_dev_{}", i), variance.sqrt());
        }

        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_record() -> DataRecord {
        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), "test".to_string());
        
        DataRecord {
            id: 1,
            values: vec![10.0, 20.0, 30.0],
            metadata,
        }
    }

    #[test]
    fn test_valid_record() {
        let processor = DataProcessor::new(0.0, 100.0, vec!["source".to_string()]);
        let record = create_test_record();
        
        assert!(processor.validate_record(&record).is_ok());
    }

    #[test]
    fn test_invalid_id() {
        let processor = DataProcessor::new(0.0, 100.0, vec!["source".to_string()]);
        let mut record = create_test_record();
        record.id = 0;
        
        assert!(matches!(
            processor.validate_record(&record),
            Err(ValidationError::InvalidId)
        ));
    }

    #[test]
    fn test_normalization() {
        let processor = DataProcessor::new(0.0, 100.0, vec!["source".to_string()]);
        let mut record = create_test_record();
        
        processor.normalize_values(&mut record);
        
        assert_eq!(record.values[0], 0.0);
        assert_eq!(record.values[1], 0.5);
        assert_eq!(record.values[2], 1.0);
    }

    #[test]
    fn test_statistics_calculation() {
        let processor = DataProcessor::new(0.0, 100.0, vec!["source".to_string()]);
        let records = vec![create_test_record(), create_test_record()];
        
        let stats = processor.calculate_statistics(&records);
        
        assert_eq!(stats.get("mean_0"), Some(&10.0));
        assert_eq!(stats.get("variance_0"), Some(&0.0));
    }
}