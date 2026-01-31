
use std::collections::HashMap;
use std::error::Error;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub category: String,
}

#[derive(Debug)]
pub struct DataProcessor {
    records: Vec<DataRecord>,
    statistics: HashMap<String, f64>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
            statistics: HashMap::new(),
        }
    }

    pub fn add_record(&mut self, record: DataRecord) -> Result<(), Box<dyn Error>> {
        if record.value < 0.0 {
            return Err("Value cannot be negative".into());
        }
        
        if record.name.is_empty() {
            return Err("Name cannot be empty".into());
        }

        self.records.push(record);
        self.update_statistics();
        Ok(())
    }

    pub fn process_records(&mut self) -> HashMap<String, f64> {
        let mut results = HashMap::new();
        
        if self.records.is_empty() {
            return results;
        }

        let total: f64 = self.records.iter().map(|r| r.value).sum();
        let count = self.records.len() as f64;
        let average = total / count;

        let max_value = self.records
            .iter()
            .map(|r| r.value)
            .fold(f64::MIN, |a, b| a.max(b));

        let min_value = self.records
            .iter()
            .map(|r| r.value)
            .fold(f64::MAX, |a, b| a.min(b));

        results.insert("total".to_string(), total);
        results.insert("average".to_string(), average);
        results.insert("max".to_string(), max_value);
        results.insert("min".to_string(), min_value);
        results.insert("count".to_string(), count);

        self.statistics = results.clone();
        results
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<DataRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .cloned()
            .collect()
    }

    pub fn transform_values<F>(&mut self, transform_fn: F) 
    where
        F: Fn(f64) -> f64,
    {
        for record in &mut self.records {
            record.value = transform_fn(record.value);
        }
        self.update_statistics();
    }

    fn update_statistics(&mut self) {
        if !self.records.is_empty() {
            self.process_records();
        }
    }

    pub fn get_statistics(&self) -> &HashMap<String, f64> {
        &self.statistics
    }

    pub fn export_json(&self) -> Result<String, Box<dyn Error>> {
        let json = serde_json::to_string_pretty(&self.records)?;
        Ok(json)
    }

    pub fn import_json(&mut self, json_data: &str) -> Result<(), Box<dyn Error>> {
        let records: Vec<DataRecord> = serde_json::from_str(json_data)?;
        self.records = records;
        self.update_statistics();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_valid_record() {
        let mut processor = DataProcessor::new();
        let record = DataRecord {
            id: 1,
            name: "Test".to_string(),
            value: 100.0,
            category: "A".to_string(),
        };
        
        assert!(processor.add_record(record).is_ok());
        assert_eq!(processor.get_statistics().get("count"), Some(&1.0));
    }

    #[test]
    fn test_add_invalid_record() {
        let mut processor = DataProcessor::new();
        let record = DataRecord {
            id: 1,
            name: "".to_string(),
            value: -50.0,
            category: "A".to_string(),
        };
        
        assert!(processor.add_record(record).is_err());
    }

    #[test]
    fn test_process_records() {
        let mut processor = DataProcessor::new();
        
        let records = vec![
            DataRecord { id: 1, name: "A".to_string(), value: 10.0, category: "X".to_string() },
            DataRecord { id: 2, name: "B".to_string(), value: 20.0, category: "X".to_string() },
            DataRecord { id: 3, name: "C".to_string(), value: 30.0, category: "Y".to_string() },
        ];

        for record in records {
            processor.add_record(record).unwrap();
        }

        let stats = processor.process_records();
        assert_eq!(stats.get("total"), Some(&60.0));
        assert_eq!(stats.get("average"), Some(&20.0));
        assert_eq!(stats.get("count"), Some(&3.0));
    }

    #[test]
    fn test_filter_by_category() {
        let mut processor = DataProcessor::new();
        
        let records = vec![
            DataRecord { id: 1, name: "A".to_string(), value: 10.0, category: "X".to_string() },
            DataRecord { id: 2, name: "B".to_string(), value: 20.0, category: "Y".to_string() },
            DataRecord { id: 3, name: "C".to_string(), value: 30.0, category: "X".to_string() },
        ];

        for record in records {
            processor.add_record(record).unwrap();
        }

        let filtered = processor.filter_by_category("X");
        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().all(|r| r.category == "X"));
    }
}