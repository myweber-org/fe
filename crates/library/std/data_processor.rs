
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
            ValidationError::InvalidId => write!(f, "ID must be greater than 0"),
            ValidationError::EmptyValues => write!(f, "Values vector cannot be empty"),
            ValidationError::ValueOutOfRange(val) => write!(f, "Value {} is out of valid range", val),
            ValidationError::MissingMetadata(key) => write!(f, "Missing required metadata: {}", key),
        }
    }
}

impl Error for ValidationError {}

impl DataRecord {
    pub fn new(id: u32, values: Vec<f64>, metadata: HashMap<String, String>) -> Self {
        Self {
            id,
            values,
            metadata,
        }
    }

    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.id == 0 {
            return Err(ValidationError::InvalidId);
        }

        if self.values.is_empty() {
            return Err(ValidationError::EmptyValues);
        }

        for &value in &self.values {
            if !value.is_finite() || value < 0.0 || value > 1000.0 {
                return Err(ValidationError::ValueOutOfRange(value));
            }
        }

        let required_keys = ["source", "timestamp"];
        for key in required_keys.iter() {
            if !self.metadata.contains_key(*key) {
                return Err(ValidationError::MissingMetadata(key.to_string()));
            }
        }

        Ok(())
    }

    pub fn normalize_values(&mut self) {
        if let Some(max_value) = self.values.iter().copied().reduce(f64::max) {
            if max_value > 0.0 {
                for value in &mut self.values {
                    *value /= max_value;
                }
            }
        }
    }

    pub fn calculate_statistics(&self) -> HashMap<String, f64> {
        let mut stats = HashMap::new();

        if self.values.is_empty() {
            return stats;
        }

        let sum: f64 = self.values.iter().sum();
        let count = self.values.len() as f64;
        let mean = sum / count;

        let variance: f64 = self.values
            .iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / count;

        stats.insert("mean".to_string(), mean);
        stats.insert("sum".to_string(), sum);
        stats.insert("count".to_string(), count);
        stats.insert("variance".to_string(), variance);
        stats.insert("min".to_string(), self.values.iter().copied().reduce(f64::min).unwrap());
        stats.insert("max".to_string(), self.values.iter().copied().reduce(f64::max).unwrap());

        stats
    }
}

pub fn process_records(records: &mut [DataRecord]) -> Result<Vec<HashMap<String, f64>>, ValidationError> {
    let mut results = Vec::new();

    for record in records {
        record.validate()?;
        record.normalize_values();
        results.push(record.calculate_statistics());
    }

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_record() {
        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), "sensor_a".to_string());
        metadata.insert("timestamp".to_string(), "2024-01-15T10:30:00Z".to_string());

        let mut record = DataRecord::new(1, vec![10.0, 20.0, 30.0], metadata);
        
        assert!(record.validate().is_ok());
        
        record.normalize_values();
        assert_eq!(record.values, vec![0.3333333333333333, 0.6666666666666666, 1.0]);
        
        let stats = record.calculate_statistics();
        assert!((stats["mean"] - 0.6666666666666666).abs() < 0.0001);
    }

    #[test]
    fn test_invalid_id() {
        let metadata = HashMap::new();
        let record = DataRecord::new(0, vec![1.0, 2.0], metadata);
        
        assert!(matches!(record.validate(), Err(ValidationError::InvalidId)));
    }

    #[test]
    fn test_missing_metadata() {
        let metadata = HashMap::new();
        let record = DataRecord::new(1, vec![1.0, 2.0], metadata);
        
        assert!(matches!(record.validate(), Err(ValidationError::MissingMetadata(_))));
    }
}