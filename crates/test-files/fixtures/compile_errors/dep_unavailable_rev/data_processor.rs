
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DataError {
    #[error("Invalid data format")]
    InvalidFormat,
    #[error("Missing required field: {0}")]
    MissingField(String),
    #[error("Validation failed: {0}")]
    ValidationFailed(String),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DataRecord {
    pub id: u64,
    pub timestamp: i64,
    pub values: HashMap<String, f64>,
    pub tags: Vec<String>,
}

impl DataRecord {
    pub fn validate(&self) -> Result<(), DataError> {
        if self.id == 0 {
            return Err(DataError::ValidationFailed("ID cannot be zero".to_string()));
        }
        
        if self.timestamp < 0 {
            return Err(DataError::ValidationFailed("Timestamp cannot be negative".to_string()));
        }
        
        if self.values.is_empty() {
            return Err(DataError::ValidationFailed("Values cannot be empty".to_string()));
        }
        
        Ok(())
    }
    
    pub fn transform(&mut self, multiplier: f64) {
        for value in self.values.values_mut() {
            *value *= multiplier;
        }
    }
    
    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
    }
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }
    
    pub fn add_record(&mut self, record: DataRecord) -> Result<(), DataError> {
        record.validate()?;
        self.records.push(record);
        Ok(())
    }
    
    pub fn process_all(&mut self, multiplier: f64) {
        for record in &mut self.records {
            record.transform(multiplier);
        }
    }
    
    pub fn filter_by_tag(&self, tag: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.tags.contains(&tag.to_string()))
            .collect()
    }
    
    pub fn calculate_average(&self, key: &str) -> Option<f64> {
        let values: Vec<f64> = self.records
            .iter()
            .filter_map(|record| record.values.get(key).copied())
            .collect();
        
        if values.is_empty() {
            None
        } else {
            let sum: f64 = values.iter().sum();
            Some(sum / values.len() as f64)
        }
    }
    
    pub fn get_statistics(&self) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        
        for record in &self.records {
            for (key, value) in &record.values {
                let entry = stats.entry(key.clone()).or_insert(0.0);
                *entry += value;
            }
        }
        
        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_record_validation() {
        let mut values = HashMap::new();
        values.insert("temperature".to_string(), 25.5);
        
        let valid_record = DataRecord {
            id: 1,
            timestamp: 1625097600,
            values: values.clone(),
            tags: vec!["sensor".to_string()],
        };
        
        assert!(valid_record.validate().is_ok());
        
        let invalid_record = DataRecord {
            id: 0,
            timestamp: 1625097600,
            values: values,
            tags: vec![],
        };
        
        assert!(invalid_record.validate().is_err());
    }
    
    #[test]
    fn test_data_processing() {
        let mut processor = DataProcessor::new();
        
        let mut values = HashMap::new();
        values.insert("pressure".to_string(), 100.0);
        
        let record = DataRecord {
            id: 1,
            timestamp: 1625097600,
            values,
            tags: vec!["test".to_string()],
        };
        
        assert!(processor.add_record(record).is_ok());
        processor.process_all(2.0);
        
        let filtered = processor.filter_by_tag("test");
        assert_eq!(filtered.len(), 1);
        
        let avg = processor.calculate_average("pressure");
        assert_eq!(avg, Some(200.0));
    }
}