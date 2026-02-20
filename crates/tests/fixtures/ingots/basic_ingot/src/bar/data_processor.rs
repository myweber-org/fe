use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRecord {
    pub id: u64,
    pub timestamp: i64,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, PartialEq)]
pub enum ValidationError {
    InvalidId,
    InvalidTimestamp,
    EmptyValues,
    InvalidMetadata,
}

impl DataRecord {
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.id == 0 {
            return Err(ValidationError::InvalidId);
        }
        
        if self.timestamp <= 0 {
            return Err(ValidationError::InvalidTimestamp);
        }
        
        if self.values.is_empty() {
            return Err(ValidationError::EmptyValues);
        }
        
        if self.metadata.contains_key("") {
            return Err(ValidationError::InvalidMetadata);
        }
        
        Ok(())
    }
    
    pub fn transform(&mut self, factor: f64) -> &Self {
        for value in self.values.iter_mut() {
            *value *= factor;
        }
        self
    }
    
    pub fn calculate_statistics(&self) -> (f64, f64, f64) {
        let count = self.values.len() as f64;
        let sum: f64 = self.values.iter().sum();
        let mean = sum / count;
        
        let variance: f64 = self.values
            .iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / count;
        
        let std_dev = variance.sqrt();
        
        (mean, variance, std_dev)
    }
}

pub fn process_records(records: &mut [DataRecord], factor: f64) -> Result<Vec<DataRecord>, Box<dyn Error>> {
    let mut processed = Vec::with_capacity(records.len());
    
    for record in records.iter_mut() {
        record.validate()?;
        let mut transformed = record.clone();
        transformed.transform(factor);
        processed.push(transformed);
    }
    
    Ok(processed)
}

pub fn filter_records(records: &[DataRecord], threshold: f64) -> Vec<&DataRecord> {
    records
        .iter()
        .filter(|record| {
            let (mean, _, _) = record.calculate_statistics();
            mean >= threshold
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_validation() {
        let valid_record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: vec![1.0, 2.0, 3.0],
            metadata: HashMap::from([("key".to_string(), "value".to_string())]),
        };
        
        assert!(valid_record.validate().is_ok());
        
        let invalid_record = DataRecord {
            id: 0,
            timestamp: 1234567890,
            values: vec![1.0, 2.0, 3.0],
            metadata: HashMap::new(),
        };
        
        assert_eq!(invalid_record.validate(), Err(ValidationError::InvalidId));
    }
    
    #[test]
    fn test_transform() {
        let mut record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: vec![1.0, 2.0, 3.0],
            metadata: HashMap::new(),
        };
        
        record.transform(2.0);
        assert_eq!(record.values, vec![2.0, 4.0, 6.0]);
    }
    
    #[test]
    fn test_statistics() {
        let record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: vec![1.0, 2.0, 3.0, 4.0, 5.0],
            metadata: HashMap::new(),
        };
        
        let (mean, variance, std_dev) = record.calculate_statistics();
        
        assert_eq!(mean, 3.0);
        assert_eq!(variance, 2.0);
        assert_eq!(std_dev, 2.0_f64.sqrt());
    }
}