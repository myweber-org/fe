
use std::collections::HashMap;

#[derive(Debug, Clone)]
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

    pub fn add_value(&mut self, value: f64) -> &mut Self {
        self.values.push(value);
        self
    }

    pub fn add_metadata(&mut self, key: &str, value: &str) -> &mut Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
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

    pub fn calculate_statistics(&self) -> Option<DataStatistics> {
        if self.values.is_empty() {
            return None;
        }

        let count = self.values.len();
        let sum: f64 = self.values.iter().sum();
        let mean = sum / count as f64;
        let min = self.values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = self.values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

        let variance: f64 = self.values
            .iter()
            .map(|value| {
                let diff = mean - value;
                diff * diff
            })
            .sum::<f64>() / count as f64;

        Some(DataStatistics {
            count,
            sum,
            mean,
            min,
            max,
            variance,
            std_dev: variance.sqrt(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct DataStatistics {
    pub count: usize,
    pub sum: f64,
    pub mean: f64,
    pub min: f64,
    pub max: f64,
    pub variance: f64,
    pub std_dev: f64,
}

pub fn process_records(records: &[DataRecord]) -> Vec<Result<DataStatistics, String>> {
    records
        .iter()
        .map(|record| {
            record.validate()?;
            record.calculate_statistics()
                .ok_or_else(|| "Failed to calculate statistics".to_string())
        })
        .collect()
}

pub fn filter_valid_records(records: &[DataRecord]) -> Vec<&DataRecord> {
    records
        .iter()
        .filter(|record| record.validate().is_ok())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_creation() {
        let mut record = DataRecord::new(1, 1234567890);
        record.add_value(42.5).add_value(37.2);
        record.add_metadata("source", "sensor_a");

        assert_eq!(record.id, 1);
        assert_eq!(record.timestamp, 1234567890);
        assert_eq!(record.values.len(), 2);
        assert_eq!(record.metadata.get("source"), Some(&"sensor_a".to_string()));
    }

    #[test]
    fn test_record_validation() {
        let valid_record = DataRecord::new(1, 1234567890).add_value(42.5);
        assert!(valid_record.validate().is_ok());

        let invalid_id = DataRecord::new(0, 1234567890).add_value(42.5);
        assert!(invalid_id.validate().is_err());

        let invalid_timestamp = DataRecord::new(1, -1).add_value(42.5);
        assert!(invalid_timestamp.validate().is_err());

        let no_values = DataRecord::new(1, 1234567890);
        assert!(no_values.validate().is_err());
    }

    #[test]
    fn test_statistics_calculation() {
        let mut record = DataRecord::new(1, 1234567890);
        record.add_value(10.0).add_value(20.0).add_value(30.0);

        let stats = record.calculate_statistics().unwrap();
        assert_eq!(stats.count, 3);
        assert_eq!(stats.sum, 60.0);
        assert_eq!(stats.mean, 20.0);
        assert_eq!(stats.min, 10.0);
        assert_eq!(stats.max, 30.0);
    }

    #[test]
    fn test_process_records() {
        let mut record1 = DataRecord::new(1, 1234567890);
        record1.add_value(10.0).add_value(20.0);

        let mut record2 = DataRecord::new(2, 1234567891);
        record2.add_value(30.0).add_value(40.0);

        let records = vec![record1, record2];
        let results = process_records(&records);

        assert_eq!(results.len(), 2);
        assert!(results[0].is_ok());
        assert!(results[1].is_ok());
    }
}