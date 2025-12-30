
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

impl DataRecord {
    pub fn new(id: u32, values: Vec<f64>) -> Self {
        Self {
            id,
            values,
            metadata: HashMap::new(),
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.id == 0 {
            return Err("Invalid record ID".to_string());
        }

        if self.values.is_empty() {
            return Err("Empty values vector".to_string());
        }

        for value in &self.values {
            if value.is_nan() || value.is_infinite() {
                return Err("Invalid numeric value detected".to_string());
            }
        }

        Ok(())
    }

    pub fn calculate_statistics(&self) -> (f64, f64, f64) {
        let sum: f64 = self.values.iter().sum();
        let count = self.values.len() as f64;
        let mean = sum / count;

        let variance: f64 = self.values
            .iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / count;

        let std_dev = variance.sqrt();

        (mean, variance, std_dev)
    }

    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }
}

pub fn process_records(records: Vec<DataRecord>) -> Vec<DataRecord> {
    records
        .into_iter()
        .filter(|record| record.validate().is_ok())
        .map(|mut record| {
            let (mean, _, std_dev) = record.calculate_statistics();
            record.add_metadata("mean".to_string(), format!("{:.4}", mean));
            record.add_metadata("std_dev".to_string(), format!("{:.4}", std_dev));
            record
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_validation() {
        let valid_record = DataRecord::new(1, vec![1.0, 2.0, 3.0]);
        assert!(valid_record.validate().is_ok());

        let invalid_record = DataRecord::new(0, vec![1.0, 2.0]);
        assert!(invalid_record.validate().is_err());
    }

    #[test]
    fn test_statistics_calculation() {
        let record = DataRecord::new(1, vec![1.0, 2.0, 3.0, 4.0, 5.0]);
        let (mean, variance, std_dev) = record.calculate_statistics();
        
        assert_eq!(mean, 3.0);
        assert_eq!(variance, 2.0);
        assert_eq!(std_dev, 2.0_f64.sqrt());
    }

    #[test]
    fn test_process_records() {
        let records = vec![
            DataRecord::new(1, vec![1.0, 2.0, 3.0]),
            DataRecord::new(0, vec![4.0, 5.0]), // Invalid ID
            DataRecord::new(2, vec![f64::NAN, 2.0]), // Invalid value
        ];

        let processed = process_records(records);
        assert_eq!(processed.len(), 1);
        assert_eq!(processed[0].id, 1);
    }
}