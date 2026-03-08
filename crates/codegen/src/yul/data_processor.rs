
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub category: String,
}

#[derive(Debug)]
pub enum ProcessingError {
    InvalidValue,
    InvalidCategory,
    EmptyData,
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessingError::InvalidValue => write!(f, "Value must be positive"),
            ProcessingError::InvalidCategory => write!(f, "Category cannot be empty"),
            ProcessingError::EmptyData => write!(f, "No data records provided"),
        }
    }
}

impl Error for ProcessingError {}

pub fn validate_record(record: &DataRecord) -> Result<(), ProcessingError> {
    if record.value <= 0.0 {
        return Err(ProcessingError::InvalidValue);
    }
    
    if record.category.trim().is_empty() {
        return Err(ProcessingError::InvalidCategory);
    }
    
    Ok(())
}

pub fn process_records(records: &[DataRecord]) -> Result<Vec<DataRecord>, ProcessingError> {
    if records.is_empty() {
        return Err(ProcessingError::EmptyData);
    }
    
    let mut processed = Vec::with_capacity(records.len());
    
    for record in records {
        validate_record(record)?;
        
        let processed_record = DataRecord {
            id: record.id,
            value: record.value * 1.1,
            category: record.category.to_uppercase(),
        };
        
        processed.push(processed_record);
    }
    
    Ok(processed)
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
    fn test_validate_record_valid() {
        let record = DataRecord {
            id: 1,
            value: 42.5,
            category: "test".to_string(),
        };
        
        assert!(validate_record(&record).is_ok());
    }
    
    #[test]
    fn test_validate_record_invalid_value() {
        let record = DataRecord {
            id: 1,
            value: -5.0,
            category: "test".to_string(),
        };
        
        assert!(matches!(validate_record(&record), Err(ProcessingError::InvalidValue)));
    }
    
    #[test]
    fn test_process_records() {
        let records = vec![
            DataRecord { id: 1, value: 10.0, category: "alpha".to_string() },
            DataRecord { id: 2, value: 20.0, category: "beta".to_string() },
        ];
        
        let processed = process_records(&records).unwrap();
        
        assert_eq!(processed[0].value, 11.0);
        assert_eq!(processed[0].category, "ALPHA");
        assert_eq!(processed[1].value, 22.0);
        assert_eq!(processed[1].category, "BETA");
    }
    
    #[test]
    fn test_calculate_statistics() {
        let records = vec![
            DataRecord { id: 1, value: 10.0, category: "test".to_string() },
            DataRecord { id: 2, value: 20.0, category: "test".to_string() },
            DataRecord { id: 3, value: 30.0, category: "test".to_string() },
        ];
        
        let (mean, variance, std_dev) = calculate_statistics(&records);
        
        assert!((mean - 20.0).abs() < 0.001);
        assert!((variance - 66.666).abs() < 0.001);
        assert!((std_dev - 8.1649).abs() < 0.001);
    }
}