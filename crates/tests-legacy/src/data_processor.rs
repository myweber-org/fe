use std::collections::HashMap;
use std::error::Error;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub timestamp: i64,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

impl DataRecord {
    pub fn new(id: u32, timestamp: i64, values: Vec<f64>) -> Self {
        Self {
            id,
            timestamp,
            values,
            metadata: HashMap::new(),
        }
    }

    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    pub fn validate(&self) -> Result<(), Box<dyn Error>> {
        if self.id == 0 {
            return Err("Invalid record ID".into());
        }
        
        if self.timestamp < 0 {
            return Err("Invalid timestamp".into());
        }
        
        if self.values.is_empty() {
            return Err("Empty values array".into());
        }
        
        for value in &self.values {
            if value.is_nan() || value.is_infinite() {
                return Err("Invalid numeric value".into());
            }
        }
        
        Ok(())
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

pub fn process_records(records: &[DataRecord]) -> Result<Vec<DataRecord>, Box<dyn Error>> {
    let mut processed = Vec::with_capacity(records.len());
    
    for record in records {
        record.validate()?;
        
        let mut processed_record = record.clone();
        processed_record.values = normalize_values(&record.values);
        processed_record.add_metadata(
            "processed".to_string(),
            "true".to_string()
        );
        
        processed.push(processed_record);
    }
    
    Ok(processed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_validation() {
        let valid_record = DataRecord::new(1, 1234567890, vec![1.0, 2.0, 3.0]);
        assert!(valid_record.validate().is_ok());
        
        let invalid_record = DataRecord::new(0, 1234567890, vec![1.0, 2.0]);
        assert!(invalid_record.validate().is_err());
    }
    
    #[test]
    fn test_normalize_values() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let normalized = normalize_values(&values);
        
        assert_eq!(normalized.len(), 5);
        assert!((normalized[0] - 0.0).abs() < 0.001);
        assert!((normalized[4] - 1.0).abs() < 0.001);
    }
    
    #[test]
    fn test_process_records() {
        let records = vec![
            DataRecord::new(1, 1000, vec![10.0, 20.0]),
            DataRecord::new(2, 2000, vec![30.0, 40.0]),
        ];
        
        let processed = process_records(&records).unwrap();
        assert_eq!(processed.len(), 2);
        assert_eq!(processed[0].metadata.get("processed"), Some(&"true".to_string()));
    }
}