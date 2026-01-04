
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct DataRecord {
    id: u32,
    name: String,
    value: f64,
    tags: Vec<String>,
}

impl DataRecord {
    pub fn new(id: u32, name: String, value: f64, tags: Vec<String>) -> Self {
        Self { id, name, value, tags }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("Name cannot be empty".to_string());
        }
        if self.value < 0.0 {
            return Err("Value must be non-negative".to_string());
        }
        Ok(())
    }

    pub fn transform(&mut self, multiplier: f64) {
        self.value *= multiplier;
        self.name = self.name.to_uppercase();
    }
}

pub struct DataProcessor {
    records: HashMap<u32, DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        Self {
            records: HashMap::new(),
        }
    }

    pub fn add_record(&mut self, record: DataRecord) -> Result<(), String> {
        record.validate()?;
        self.records.insert(record.id, record);
        Ok(())
    }

    pub fn process_records(&mut self, multiplier: f64) {
        for record in self.records.values_mut() {
            record.transform(multiplier);
        }
    }

    pub fn get_record(&self, id: u32) -> Option<&DataRecord> {
        self.records.get(&id)
    }

    pub fn total_value(&self) -> f64 {
        self.records.values().map(|r| r.value).sum()
    }

    pub fn filter_by_tag(&self, tag: &str) -> Vec<&DataRecord> {
        self.records
            .values()
            .filter(|r| r.tags.contains(&tag.to_string()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_validation() {
        let valid_record = DataRecord::new(1, "test".to_string(), 10.0, vec![]);
        assert!(valid_record.validate().is_ok());

        let invalid_record = DataRecord::new(2, "".to_string(), -5.0, vec![]);
        assert!(invalid_record.validate().is_err());
    }

    #[test]
    fn test_data_processing() {
        let mut processor = DataProcessor::new();
        let record = DataRecord::new(1, "sample".to_string(), 100.0, vec!["tag1".to_string()]);
        
        assert!(processor.add_record(record).is_ok());
        processor.process_records(2.0);
        
        let processed = processor.get_record(1).unwrap();
        assert_eq!(processed.value, 200.0);
        assert_eq!(processed.name, "SAMPLE");
    }
}