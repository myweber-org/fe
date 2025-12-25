
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct DataRecord {
    id: u32,
    category: String,
    value: f64,
    metadata: HashMap<String, String>,
}

impl DataRecord {
    pub fn new(id: u32, category: String, value: f64) -> Self {
        Self {
            id,
            category,
            value,
            metadata: HashMap::new(),
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.id == 0 {
            return Err("ID cannot be zero".to_string());
        }
        if self.category.is_empty() {
            return Err("Category cannot be empty".to_string());
        }
        if self.value < 0.0 {
            return Err("Value cannot be negative".to_string());
        }
        Ok(())
    }

    pub fn transform(&mut self, multiplier: f64) {
        self.value *= multiplier;
    }

    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }
}

pub fn process_records(records: &mut [DataRecord], multiplier: f64) -> Result<(), String> {
    for record in records.iter_mut() {
        record.validate()?;
        record.transform(multiplier);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_validation() {
        let valid_record = DataRecord::new(1, "test".to_string(), 100.0);
        assert!(valid_record.validate().is_ok());

        let invalid_record = DataRecord::new(0, "test".to_string(), 100.0);
        assert!(invalid_record.validate().is_err());
    }

    #[test]
    fn test_record_transformation() {
        let mut record = DataRecord::new(1, "test".to_string(), 100.0);
        record.transform(2.0);
        assert_eq!(record.value, 200.0);
    }

    #[test]
    fn test_metadata_addition() {
        let mut record = DataRecord::new(1, "test".to_string(), 100.0);
        record.add_metadata("source".to_string(), "api".to_string());
        assert_eq!(record.metadata.get("source"), Some(&"api".to_string()));
    }
}