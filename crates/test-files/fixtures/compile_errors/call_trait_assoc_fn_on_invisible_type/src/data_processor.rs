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

    pub fn transform(&mut self, factor: f64) {
        for value in &mut self.values {
            *value *= factor;
        }
    }

    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }
}

pub fn process_records(records: &mut [DataRecord], factor: f64) -> Vec<DataRecord> {
    records
        .iter_mut()
        .filter(|record| record.is_valid())
        .map(|record| {
            let mut transformed = record.clone();
            transformed.transform(factor);
            transformed.add_metadata(
                "processed".to_string(),
                format!("transformed_with_factor_{}", factor),
            );
            transformed
        })
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
    fn test_record_transformation() {
        let mut record = DataRecord::new(1, vec![1.0, 2.0, 3.0]);
        record.transform(2.0);
        assert_eq!(record.values, vec![2.0, 4.0, 6.0]);
    }

    #[test]
    fn test_process_records() {
        let mut records = vec![
            DataRecord::new(1, vec![1.0, 2.0]),
            DataRecord::new(0, vec![]),
            DataRecord::new(2, vec![3.0, 4.0]),
        ];

        let processed = process_records(&mut records, 3.0);
        assert_eq!(processed.len(), 2);
        assert_eq!(processed[0].values, vec![3.0, 6.0]);
        assert_eq!(processed[1].values, vec![9.0, 12.0]);
    }
}