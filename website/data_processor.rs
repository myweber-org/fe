
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
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

    pub fn add_metadata(&mut self, key: &str, value: &str) {
        self.metadata.insert(key.to_string(), value.to_string());
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.id == 0 {
            return Err("Invalid record ID".to_string());
        }
        
        if self.timestamp < 0 {
            return Err("Timestamp cannot be negative".to_string());
        }
        
        if self.values.is_empty() {
            return Err("Values array cannot be empty".to_string());
        }
        
        for value in &self.values {
            if !value.is_finite() {
                return Err("Values must be finite numbers".to_string());
            }
        }
        
        Ok(())
    }

    pub fn transform_values<F>(&mut self, transformer: F)
    where
        F: Fn(f64) -> f64,
    {
        self.values = self.values.iter().map(|&v| transformer(v)).collect();
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
            .map(|&v| (v - mean).powi(2))
            .sum::<f64>() / count;
        
        let min = self.values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = self.values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        
        stats.insert("mean".to_string(), mean);
        stats.insert("variance".to_string(), variance);
        stats.insert("min".to_string(), min);
        stats.insert("max".to_string(), max);
        stats.insert("count".to_string(), count);
        
        stats
    }
}

pub fn process_records(records: &mut [DataRecord]) -> Result<Vec<HashMap<String, f64>>, String> {
    let mut results = Vec::new();
    
    for record in records {
        record.validate()?;
        record.transform_values(|v| v * 2.0);
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
    fn test_value_transformation() {
        let mut record = DataRecord::new(1, 1234567890, vec![1.0, 2.0, 3.0]);
        record.transform_values(|v| v * 2.0);
        assert_eq!(record.values, vec![2.0, 4.0, 6.0]);
    }

    #[test]
    fn test_statistics_calculation() {
        let record = DataRecord::new(1, 1234567890, vec![1.0, 2.0, 3.0, 4.0]);
        let stats = record.calculate_statistics();
        
        assert_eq!(stats.get("mean"), Some(&2.5));
        assert_eq!(stats.get("count"), Some(&4.0));
        assert_eq!(stats.get("min"), Some(&1.0));
        assert_eq!(stats.get("max"), Some(&4.0));
    }
}