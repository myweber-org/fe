
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

pub fn process_records(records: &mut [DataRecord], multiplier: f64) -> Result<Vec<DataRecord>, DataError> {
    let mut processed = Vec::with_capacity(records.len());
    
    for record in records.iter_mut() {
        record.validate()?;
        record.transform(multiplier);
        record.add_tag("processed".to_string());
        processed.push(record.clone());
    }
    
    Ok(processed)
}

pub fn calculate_statistics(records: &[DataRecord]) -> HashMap<String, f64> {
    let mut stats = HashMap::new();
    
    if records.is_empty() {
        return stats;
    }
    
    for key in records[0].values.keys() {
        let values: Vec<f64> = records.iter()
            .filter_map(|r| r.values.get(key).copied())
            .collect();
        
        if !values.is_empty() {
            let sum: f64 = values.iter().sum();
            let count = values.len() as f64;
            let avg = sum / count;
            
            let variance: f64 = values.iter()
                .map(|v| (v - avg).powi(2))
                .sum::<f64>() / count;
            
            stats.insert(format!("{}_avg", key), avg);
            stats.insert(format!("{}_variance", key), variance);
        }
    }
    
    stats
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_record_validation() {
        let mut record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: HashMap::from([("temperature".to_string(), 25.5)]),
            tags: vec![],
        };
        
        assert!(record.validate().is_ok());
        
        record.id = 0;
        assert!(record.validate().is_err());
    }
    
    #[test]
    fn test_record_transformation() {
        let mut record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: HashMap::from([("value".to_string(), 10.0)]),
            tags: vec![],
        };
        
        record.transform(2.0);
        assert_eq!(record.values.get("value"), Some(&20.0));
    }
}use std::collections::HashMap;

pub struct DataProcessor {
    cache: HashMap<String, Vec<f64>>,
    validation_rules: Vec<ValidationRule>,
}

pub struct ValidationRule {
    field_name: String,
    min_value: f64,
    max_value: f64,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            cache: HashMap::new(),
            validation_rules: Vec::new(),
        }
    }

    pub fn add_validation_rule(&mut self, field: &str, min: f64, max: f64) {
        self.validation_rules.push(ValidationRule {
            field_name: field.to_string(),
            min_value: min,
            max_value: max,
        });
    }

    pub fn process_dataset(&mut self, dataset: &[HashMap<String, f64>]) -> Result<Vec<HashMap<String, f64>>, String> {
        let mut results = Vec::new();
        
        for (index, record) in dataset.iter().enumerate() {
            match self.validate_record(record) {
                Ok(validated) => {
                    let transformed = self.transform_record(&validated);
                    self.cache.insert(format!("record_{}", index), transformed.values().cloned().collect());
                    results.push(transformed);
                }
                Err(e) => return Err(format!("Validation failed at record {}: {}", index, e)),
            }
        }
        
        Ok(results)
    }

    fn validate_record(&self, record: &HashMap<String, f64>) -> Result<HashMap<String, f64>, String> {
        for rule in &self.validation_rules {
            if let Some(&value) = record.get(&rule.field_name) {
                if value < rule.min_value || value > rule.max_value {
                    return Err(format!("Field '{}' value {} out of range [{}, {}]", 
                        rule.field_name, value, rule.min_value, rule.max_value));
                }
            }
        }
        Ok(record.clone())
    }

    fn transform_record(&self, record: &HashMap<String, f64>) -> HashMap<String, f64> {
        let mut transformed = HashMap::new();
        
        for (key, value) in record {
            let new_key = format!("processed_{}", key);
            let new_value = if key.contains("normalized") {
                *value
            } else {
                value * 2.0
            };
            transformed.insert(new_key, new_value);
        }
        
        transformed
    }

    pub fn get_cached_values(&self, key: &str) -> Option<&Vec<f64>> {
        self.cache.get(key)
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_processing() {
        let mut processor = DataProcessor::new();
        processor.add_validation_rule("temperature", -50.0, 150.0);
        
        let test_data = vec![
            [("temperature".to_string(), 25.0)].iter().cloned().collect(),
            [("temperature".to_string(), 100.0)].iter().cloned().collect(),
        ];
        
        let result = processor.process_dataset(&test_data);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 2);
    }

    #[test]
    fn test_validation_failure() {
        let mut processor = DataProcessor::new();
        processor.add_validation_rule("pressure", 0.0, 100.0);
        
        let invalid_data = vec![
            [("pressure".to_string(), 150.0)].iter().cloned().collect(),
        ];
        
        let result = processor.process_dataset(&invalid_data);
        assert!(result.is_err());
    }
}