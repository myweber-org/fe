
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DataError {
    #[error("Invalid input data")]
    InvalidInput,
    #[error("Processing timeout")]
    Timeout,
    #[error("Serialization failed: {0}")]
    Serialization(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRecord {
    pub id: u64,
    pub timestamp: i64,
    pub values: HashMap<String, f64>,
    pub tags: Vec<String>,
}

impl DataRecord {
    pub fn new(id: u64, timestamp: i64) -> Self {
        Self {
            id,
            timestamp,
            values: HashMap::new(),
            tags: Vec::new(),
        }
    }

    pub fn add_value(&mut self, key: &str, value: f64) {
        self.values.insert(key.to_string(), value);
    }

    pub fn add_tag(&mut self, tag: &str) {
        self.tags.push(tag.to_string());
    }

    pub fn validate(&self) -> Result<(), DataError> {
        if self.id == 0 {
            return Err(DataError::InvalidInput);
        }
        if self.timestamp < 0 {
            return Err(DataError::InvalidInput);
        }
        if self.values.is_empty() {
            return Err(DataError::InvalidInput);
        }
        Ok(())
    }
}

pub struct DataProcessor {
    max_records: usize,
    processing_timeout: u64,
}

impl DataProcessor {
    pub fn new(max_records: usize, processing_timeout: u64) -> Self {
        Self {
            max_records,
            processing_timeout,
        }
    }

    pub fn process_records(
        &self,
        records: Vec<DataRecord>,
    ) -> Result<Vec<DataRecord>, DataError> {
        if records.len() > self.max_records {
            return Err(DataError::InvalidInput);
        }

        let mut processed = Vec::with_capacity(records.len());
        for record in records {
            record.validate()?;
            let mut processed_record = record.clone();
            self.transform_values(&mut processed_record);
            processed.push(processed_record);
        }

        Ok(processed)
    }

    fn transform_values(&self, record: &mut DataRecord) {
        let transformed: HashMap<String, f64> = record
            .values
            .iter()
            .map(|(k, v)| (k.clone(), v * 2.0))
            .collect();
        record.values = transformed;
    }

    pub fn serialize_records(&self, records: &[DataRecord]) -> Result<String, DataError> {
        serde_json::to_string(records)
            .map_err(|e| DataError::Serialization(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_validation() {
        let mut record = DataRecord::new(1, 1234567890);
        record.add_value("temperature", 23.5);
        assert!(record.validate().is_ok());

        let invalid_record = DataRecord::new(0, 1234567890);
        assert!(invalid_record.validate().is_err());
    }

    #[test]
    fn test_data_processing() {
        let processor = DataProcessor::new(100, 5000);
        let mut record = DataRecord::new(1, 1234567890);
        record.add_value("pressure", 1013.25);

        let result = processor.process_records(vec![record]);
        assert!(result.is_ok());
        let processed = result.unwrap();
        assert_eq!(processed[0].values["pressure"], 2026.5);
    }
}
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub category: String,
}

#[derive(Debug)]
pub enum ValidationError {
    InvalidId,
    InvalidName,
    InvalidValue,
    InvalidCategory,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::InvalidId => write!(f, "ID must be greater than 0"),
            ValidationError::InvalidName => write!(f, "Name cannot be empty"),
            ValidationError::InvalidValue => write!(f, "Value must be between 0.0 and 1000.0"),
            ValidationError::InvalidCategory => write!(f, "Category must be one of: A, B, C"),
        }
    }
}

impl Error for ValidationError {}

pub fn validate_record(record: &DataRecord) -> Result<(), ValidationError> {
    if record.id == 0 {
        return Err(ValidationError::InvalidId);
    }
    
    if record.name.trim().is_empty() {
        return Err(ValidationError::InvalidName);
    }
    
    if record.value < 0.0 || record.value > 1000.0 {
        return Err(ValidationError::InvalidValue);
    }
    
    let valid_categories = ["A", "B", "C"];
    if !valid_categories.contains(&record.category.as_str()) {
        return Err(ValidationError::InvalidCategory);
    }
    
    Ok(())
}

pub fn process_records(records: Vec<DataRecord>) -> Result<HashMap<String, Vec<DataRecord>>, Box<dyn Error>> {
    let mut categorized_records: HashMap<String, Vec<DataRecord>> = HashMap::new();
    
    for record in records {
        validate_record(&record)?;
        
        categorized_records
            .entry(record.category.clone())
            .or_insert_with(Vec::new)
            .push(record);
    }
    
    Ok(categorized_records)
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
            name: "Test Record".to_string(),
            value: 100.0,
            category: "A".to_string(),
        };
        
        assert!(validate_record(&record).is_ok());
    }
    
    #[test]
    fn test_validate_record_invalid_id() {
        let record = DataRecord {
            id: 0,
            name: "Test Record".to_string(),
            value: 100.0,
            category: "A".to_string(),
        };
        
        assert!(validate_record(&record).is_err());
    }
    
    #[test]
    fn test_process_records() {
        let records = vec![
            DataRecord {
                id: 1,
                name: "Record 1".to_string(),
                value: 50.0,
                category: "A".to_string(),
            },
            DataRecord {
                id: 2,
                name: "Record 2".to_string(),
                value: 75.0,
                category: "B".to_string(),
            },
        ];
        
        let result = process_records(records);
        assert!(result.is_ok());
        
        let categorized = result.unwrap();
        assert_eq!(categorized.len(), 2);
        assert!(categorized.contains_key("A"));
        assert!(categorized.contains_key("B"));
    }
    
    #[test]
    fn test_calculate_statistics() {
        let records = vec![
            DataRecord {
                id: 1,
                name: "Record 1".to_string(),
                value: 10.0,
                category: "A".to_string(),
            },
            DataRecord {
                id: 2,
                name: "Record 2".to_string(),
                value: 20.0,
                category: "A".to_string(),
            },
            DataRecord {
                id: 3,
                name: "Record 3".to_string(),
                value: 30.0,
                category: "A".to_string(),
            },
        ];
        
        let (mean, variance, std_dev) = calculate_statistics(&records);
        
        assert_eq!(mean, 20.0);
        assert_eq!(variance, 66.66666666666667);
        assert_eq!(std_dev, 8.16496580927726);
    }
}