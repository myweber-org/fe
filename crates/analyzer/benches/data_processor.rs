
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DataRecord {
    pub id: u64,
    pub timestamp: i64,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

impl DataRecord {
    pub fn new(id: u64, timestamp: i64) -> Self {
        Self {
            id,
            timestamp,
            values: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn add_value(&mut self, value: f64) {
        self.values.push(value);
    }

    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.id == 0 {
            return Err("Invalid record ID".to_string());
        }
        if self.timestamp < 0 {
            return Err("Timestamp cannot be negative".to_string());
        }
        if self.values.is_empty() {
            return Err("Record must contain at least one value".to_string());
        }
        Ok(())
    }

    pub fn calculate_statistics(&self) -> Statistics {
        let count = self.values.len();
        let sum: f64 = self.values.iter().sum();
        let mean = if count > 0 { sum / count as f64 } else { 0.0 };
        let variance: f64 = self.values.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / count as f64;
        let std_dev = variance.sqrt();

        Statistics {
            count,
            sum,
            mean,
            variance,
            std_dev,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Statistics {
    pub count: usize,
    pub sum: f64,
    pub mean: f64,
    pub variance: f64,
    pub std_dev: f64,
}

pub fn process_records(records: Vec<DataRecord>) -> Vec<Result<DataRecord, String>> {
    records.into_iter()
        .map(|mut record| {
            match record.validate() {
                Ok(_) => {
                    if record.values.len() > 10 {
                        record.values.truncate(10);
                        record.add_metadata("truncated".to_string(), "true".to_string());
                    }
                    Ok(record)
                }
                Err(e) => Err(e),
            }
        })
        .collect()
}

pub fn filter_valid_records(records: Vec<DataRecord>) -> (Vec<DataRecord>, Vec<String>) {
    let mut valid = Vec::new();
    let mut errors = Vec::new();

    for record in records {
        match record.validate() {
            Ok(_) => valid.push(record),
            Err(e) => errors.push(format!("Record {}: {}", record.id, e)),
        }
    }

    (valid, errors)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_validation() {
        let mut record = DataRecord::new(1, 1234567890);
        record.add_value(42.0);
        assert!(record.validate().is_ok());

        let invalid_record = DataRecord::new(0, 1234567890);
        assert!(invalid_record.validate().is_err());
    }

    #[test]
    fn test_statistics_calculation() {
        let mut record = DataRecord::new(1, 1234567890);
        record.add_value(10.0);
        record.add_value(20.0);
        record.add_value(30.0);

        let stats = record.calculate_statistics();
        assert_eq!(stats.count, 3);
        assert_eq!(stats.mean, 20.0);
        assert_eq!(stats.sum, 60.0);
    }

    #[test]
    fn test_process_records() {
        let mut record = DataRecord::new(1, 1234567890);
        for i in 0..15 {
            record.add_value(i as f64);
        }

        let results = process_records(vec![record.clone()]);
        assert!(results[0].is_ok());
        let processed = results[0].as_ref().unwrap();
        assert_eq!(processed.values.len(), 10);
        assert_eq!(processed.metadata.get("truncated"), Some(&"true".to_string()));
    }
}