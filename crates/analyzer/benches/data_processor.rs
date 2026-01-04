
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

    pub fn is_valid(&self) -> bool {
        !self.values.is_empty() && self.id > 0
    }

    pub fn calculate_mean(&self) -> Option<f64> {
        if self.values.is_empty() {
            return None;
        }
        let sum: f64 = self.values.iter().sum();
        Some(sum / self.values.len() as f64)
    }

    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }
}

pub fn normalize_values(values: &[f64]) -> Vec<f64> {
    if values.is_empty() {
        return Vec::new();
    }
    
    let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
    let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
    
    if (max - min).abs() < f64::EPSILON {
        return vec![0.0; values.len()];
    }
    
    values.iter()
        .map(|&v| (v - min) / (max - min))
        .collect()
}

pub fn process_records(records: &[DataRecord]) -> Vec<DataRecord> {
    records.iter()
        .filter(|r| r.is_valid())
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_validation() {
        let valid_record = DataRecord::new(1, vec![1.0, 2.0, 3.0]);
        assert!(valid_record.is_valid());
        
        let invalid_record = DataRecord::new(0, vec![]);
        assert!(!invalid_record.is_valid());
    }

    #[test]
    fn test_mean_calculation() {
        let record = DataRecord::new(1, vec![1.0, 2.0, 3.0, 4.0]);
        assert_eq!(record.calculate_mean(), Some(2.5));
        
        let empty_record = DataRecord::new(2, vec![]);
        assert_eq!(empty_record.calculate_mean(), None);
    }

    #[test]
    fn test_normalization() {
        let values = vec![1.0, 2.0, 3.0, 4.0];
        let normalized = normalize_values(&values);
        assert_eq!(normalized, vec![0.0, 1.0/3.0, 2.0/3.0, 1.0]);
    }
}