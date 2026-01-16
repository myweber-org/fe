
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;

#[derive(Debug, Serialize, Deserialize)]
pub struct DataRecord {
    id: u64,
    timestamp: i64,
    values: Vec<f64>,
    metadata: HashMap<String, String>,
}

impl DataRecord {
    pub fn new(id: u64, timestamp: i64, values: Vec<f64>) -> Self {
        Self {
            id,
            timestamp,
            values,
            metadata: HashMap::new(),
        }
    }

    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    pub fn validate(&self) -> Result<(), Box<dyn Error>> {
        if self.id == 0 {
            return Err("Invalid record ID".into());
        }
        
        if self.timestamp < 0 {
            return Err("Timestamp cannot be negative".into());
        }
        
        if self.values.is_empty() {
            return Err("Values array cannot be empty".into());
        }
        
        for value in &self.values {
            if value.is_nan() || value.is_infinite() {
                return Err("Invalid numeric value detected".into());
            }
        }
        
        Ok(())
    }

    pub fn normalize_values(&mut self) {
        if self.values.is_empty() {
            return;
        }
        
        let sum: f64 = self.values.iter().sum();
        if sum != 0.0 {
            for value in &mut self.values {
                *value /= sum;
            }
        }
    }

    pub fn calculate_statistics(&self) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        
        if self.values.is_empty() {
            return stats;
        }
        
        let count = self.values.len() as f64;
        let sum: f64 = self.values.iter().sum();
        let mean = sum / count;
        
        let variance: f64 = self.values.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / count;
        
        stats.insert("count".to_string(), count);
        stats.insert("sum".to_string(), sum);
        stats.insert("mean".to_string(), mean);
        stats.insert("variance".to_string(), variance);
        stats.insert("std_dev".to_string(), variance.sqrt());
        
        stats
    }
}

pub fn process_records(records: &mut [DataRecord]) -> Result<Vec<HashMap<String, f64>>, Box<dyn Error>> {
    let mut results = Vec::new();
    
    for record in records.iter_mut() {
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
    fn test_record_validation() {
        let valid_record = DataRecord::new(1, 1234567890, vec![1.0, 2.0, 3.0]);
        assert!(valid_record.validate().is_ok());
        
        let invalid_record = DataRecord::new(0, 1234567890, vec![1.0, 2.0]);
        assert!(invalid_record.validate().is_err());
    }

    #[test]
    fn test_normalization() {
        let mut record = DataRecord::new(1, 1234567890, vec![1.0, 2.0, 3.0]);
        record.normalize_values();
        
        let sum: f64 = record.values.iter().sum();
        assert!((sum - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_statistics_calculation() {
        let record = DataRecord::new(1, 1234567890, vec![1.0, 2.0, 3.0, 4.0]);
        let stats = record.calculate_statistics();
        
        assert_eq!(stats.get("count"), Some(&4.0));
        assert_eq!(stats.get("sum"), Some(&10.0));
        assert_eq!(stats.get("mean"), Some(&2.5));
    }
}