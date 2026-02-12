use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Validation error for field '{}': {}", self.field, self.message)
    }
}

impl Error for ValidationError {}

#[derive(Debug, Clone, PartialEq)]
pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub timestamp: u64,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, timestamp: u64) -> Result<Self, ValidationError> {
        if id == 0 {
            return Err(ValidationError {
                field: "id".to_string(),
                message: "ID must be greater than 0".to_string(),
            });
        }
        
        if !value.is_finite() {
            return Err(ValidationError {
                field: "value".to_string(),
                message: "Value must be a finite number".to_string(),
            });
        }
        
        if timestamp == 0 {
            return Err(ValidationError {
                field: "timestamp".to_string(),
                message: "Timestamp must be greater than 0".to_string(),
            });
        }
        
        Ok(DataRecord { id, value, timestamp })
    }
}

pub fn normalize_data(records: &mut [DataRecord]) {
    if records.is_empty() {
        return;
    }
    
    let min_value = records.iter().map(|r| r.value).fold(f64::INFINITY, f64::min);
    let max_value = records.iter().map(|r| r.value).fold(f64::NEG_INFINITY, f64::max);
    
    let range = max_value - min_value;
    
    if range.abs() < f64::EPSILON {
        return;
    }
    
    for record in records.iter_mut() {
        record.value = (record.value - min_value) / range;
    }
}

pub fn filter_records(records: &[DataRecord], min_value: f64, max_value: f64) -> Vec<DataRecord> {
    records
        .iter()
        .filter(|r| r.value >= min_value && r.value <= max_value)
        .cloned()
        .collect()
}

pub fn calculate_statistics(records: &[DataRecord]) -> (f64, f64, f64) {
    if records.is_empty() {
        return (0.0, 0.0, 0.0);
    }
    
    let sum: f64 = records.iter().map(|r| r.value).sum();
    let count = records.len() as f64;
    let mean = sum / count;
    
    let variance: f64 = records.iter()
        .map(|r| (r.value - mean).powi(2))
        .sum::<f64>() / count;
    
    let std_dev = variance.sqrt();
    
    (mean, variance, std_dev)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_valid_record_creation() {
        let record = DataRecord::new(1, 42.5, 1234567890).unwrap();
        assert_eq!(record.id, 1);
        assert_eq!(record.value, 42.5);
        assert_eq!(record.timestamp, 1234567890);
    }
    
    #[test]
    fn test_invalid_id() {
        let result = DataRecord::new(0, 42.5, 1234567890);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_normalize_data() {
        let mut records = vec![
            DataRecord::new(1, 10.0, 1000).unwrap(),
            DataRecord::new(2, 20.0, 2000).unwrap(),
            DataRecord::new(3, 30.0, 3000).unwrap(),
        ];
        
        normalize_data(&mut records);
        
        assert_eq!(records[0].value, 0.0);
        assert_eq!(records[1].value, 0.5);
        assert_eq!(records[2].value, 1.0);
    }
    
    #[test]
    fn test_filter_records() {
        let records = vec![
            DataRecord::new(1, 10.0, 1000).unwrap(),
            DataRecord::new(2, 20.0, 2000).unwrap(),
            DataRecord::new(3, 30.0, 3000).unwrap(),
        ];
        
        let filtered = filter_records(&records, 15.0, 25.0);
        
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, 2);
    }
    
    #[test]
    fn test_calculate_statistics() {
        let records = vec![
            DataRecord::new(1, 10.0, 1000).unwrap(),
            DataRecord::new(2, 20.0, 2000).unwrap(),
            DataRecord::new(3, 30.0, 3000).unwrap(),
        ];
        
        let (mean, variance, std_dev) = calculate_statistics(&records);
        
        assert_eq!(mean, 20.0);
        assert_eq!(variance, 66.66666666666667);
        assert_eq!(std_dev, 8.16496580927726);
    }
}