
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    id: u32,
    values: Vec<f64>,
    metadata: HashMap<String, String>,
}

#[derive(Debug)]
pub enum ProcessingError {
    InvalidData(String),
    TransformationError(String),
    ValidationFailed(String),
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessingError::InvalidData(msg) => write!(f, "Invalid data: {}", msg),
            ProcessingError::TransformationError(msg) => write!(f, "Transformation error: {}", msg),
            ProcessingError::ValidationFailed(msg) => write!(f, "Validation failed: {}", msg),
        }
    }
}

impl Error for ProcessingError {}

impl DataRecord {
    pub fn new(id: u32, values: Vec<f64>) -> Result<Self, ProcessingError> {
        if id == 0 {
            return Err(ProcessingError::InvalidData("ID cannot be zero".to_string()));
        }
        
        if values.is_empty() {
            return Err(ProcessingError::InvalidData("Values cannot be empty".to_string()));
        }
        
        Ok(Self {
            id,
            values,
            metadata: HashMap::new(),
        })
    }
    
    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }
    
    pub fn validate(&self) -> Result<(), ProcessingError> {
        for (i, &value) in self.values.iter().enumerate() {
            if !value.is_finite() {
                return Err(ProcessingError::ValidationFailed(
                    format!("Value at index {} is not finite", i)
                ));
            }
        }
        Ok(())
    }
    
    pub fn normalize(&mut self) -> Result<(), ProcessingError> {
        self.validate()?;
        
        let sum: f64 = self.values.iter().sum();
        if sum.abs() < f64::EPSILON {
            return Err(ProcessingError::TransformationError(
                "Cannot normalize zero vector".to_string()
            ));
        }
        
        for value in &mut self.values {
            *value /= sum;
        }
        
        Ok(())
    }
    
    pub fn calculate_statistics(&self) -> Result<HashMap<String, f64>, ProcessingError> {
        self.validate()?;
        
        let count = self.values.len() as f64;
        let sum: f64 = self.values.iter().sum();
        let mean = sum / count;
        
        let variance: f64 = self.values.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / count;
        
        let mut stats = HashMap::new();
        stats.insert("count".to_string(), count);
        stats.insert("sum".to_string(), sum);
        stats.insert("mean".to_string(), mean);
        stats.insert("variance".to_string(), variance);
        stats.insert("std_dev".to_string(), variance.sqrt());
        
        Ok(stats)
    }
}

pub fn process_records(records: &mut [DataRecord]) -> Result<Vec<HashMap<String, f64>>, ProcessingError> {
    let mut results = Vec::new();
    
    for record in records {
        record.normalize()?;
        let stats = record.calculate_statistics()?;
        results.push(stats);
    }
    
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_valid_record_creation() {
        let record = DataRecord::new(1, vec![1.0, 2.0, 3.0]);
        assert!(record.is_ok());
    }
    
    #[test]
    fn test_invalid_record_zero_id() {
        let record = DataRecord::new(0, vec![1.0, 2.0]);
        assert!(record.is_err());
    }
    
    #[test]
    fn test_normalization() {
        let mut record = DataRecord::new(1, vec![1.0, 2.0, 3.0]).unwrap();
        assert!(record.normalize().is_ok());
        
        let sum: f64 = record.values.iter().sum();
        assert!((sum - 1.0).abs() < f64::EPSILON);
    }
    
    #[test]
    fn test_statistics_calculation() {
        let record = DataRecord::new(1, vec![1.0, 2.0, 3.0]).unwrap();
        let stats = record.calculate_statistics().unwrap();
        
        assert!((stats["mean"] - 2.0).abs() < f64::EPSILON);
        assert!((stats["std_dev"] - 0.816496580927726).abs() < 1e-10);
    }
}