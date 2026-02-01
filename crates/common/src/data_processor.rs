
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DataError {
    #[error("Invalid input data: {0}")]
    InvalidInput(String),
    #[error("Processing failed: {0}")]
    ProcessingFailed(String),
    #[error("Validation error: {0}")]
    ValidationError(String),
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

    pub fn add_value(&mut self, key: &str, value: f64) -> Result<(), DataError> {
        if !value.is_finite() {
            return Err(DataError::InvalidInput(
                "Value must be finite number".to_string(),
            ));
        }
        self.values.insert(key.to_string(), value);
        Ok(())
    }

    pub fn add_tag(&mut self, tag: &str) {
        if !self.tags.contains(&tag.to_string()) {
            self.tags.push(tag.to_string());
        }
    }

    pub fn validate(&self) -> Result<(), DataError> {
        if self.values.is_empty() {
            return Err(DataError::ValidationError(
                "Record must contain at least one value".to_string(),
            ));
        }

        if self.timestamp < 0 {
            return Err(DataError::ValidationError(
                "Timestamp cannot be negative".to_string(),
            ));
        }

        Ok(())
    }

    pub fn transform_values<F>(&mut self, transformer: F) -> Result<(), DataError>
    where
        F: Fn(f64) -> f64,
    {
        for value in self.values.values_mut() {
            let transformed = transformer(*value);
            if !transformed.is_finite() {
                return Err(DataError::ProcessingFailed(
                    "Transformation produced non-finite value".to_string(),
                ));
            }
            *value = transformed;
        }
        Ok(())
    }
}

pub fn process_records(records: &mut [DataRecord]) -> Result<Vec<DataRecord>, DataError> {
    let mut processed = Vec::with_capacity(records.len());

    for record in records.iter_mut() {
        record.validate()?;
        record.transform_values(|v| v * 2.0)?;
        processed.push(record.clone());
    }

    Ok(processed)
}

pub fn calculate_statistics(records: &[DataRecord]) -> HashMap<String, (f64, f64, f64)> {
    let mut stats = HashMap::new();

    for record in records {
        for (key, value) in &record.values {
            let entry = stats.entry(key.clone()).or_insert((0.0, 0.0, 0.0));
            entry.0 += value;
            entry.1 = entry.1.max(*value);
            entry.2 = if entry.2 == 0.0 {
                *value
            } else {
                entry.2.min(*value)
            };
        }
    }

    stats
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_record_creation() {
        let record = DataRecord::new(1, 1234567890);
        assert_eq!(record.id, 1);
        assert_eq!(record.timestamp, 1234567890);
        assert!(record.values.is_empty());
        assert!(record.tags.is_empty());
    }

    #[test]
    fn test_add_valid_value() {
        let mut record = DataRecord::new(1, 1234567890);
        assert!(record.add_value("temperature", 25.5).is_ok());
        assert_eq!(record.values.get("temperature"), Some(&25.5));
    }

    #[test]
    fn test_add_invalid_value() {
        let mut record = DataRecord::new(1, 1234567890);
        let result = record.add_value("invalid", f64::INFINITY);
        assert!(result.is_err());
    }

    #[test]
    fn test_record_validation() {
        let mut record = DataRecord::new(1, 1234567890);
        assert!(record.validate().is_err());

        record.add_value("test", 1.0).unwrap();
        assert!(record.validate().is_ok());
    }

    #[test]
    fn test_value_transformation() {
        let mut record = DataRecord::new(1, 1234567890);
        record.add_value("value", 10.0).unwrap();
        record.transform_values(|v| v * 2.0).unwrap();
        assert_eq!(record.values.get("value"), Some(&20.0));
    }

    #[test]
    fn test_process_records() {
        let mut records = vec![
            DataRecord::new(1, 1000),
            DataRecord::new(2, 2000),
        ];

        records[0].add_value("a", 1.0).unwrap();
        records[1].add_value("b", 2.0).unwrap();

        let processed = process_records(&mut records).unwrap();
        assert_eq!(processed.len(), 2);
        assert_eq!(processed[0].values.get("a"), Some(&2.0));
        assert_eq!(processed[1].values.get("b"), Some(&4.0));
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub timestamp: String,
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<usize, Box<dyn Error>> {
        let path = Path::new(file_path);
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        
        let mut count = 0;
        for (index, line) in reader.lines().enumerate() {
            let line = line?;
            
            if index == 0 {
                continue;
            }
            
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() != 4 {
                continue;
            }
            
            let id = match parts[0].parse::<u32>() {
                Ok(val) => val,
                Err(_) => continue,
            };
            
            let name = parts[1].to_string();
            
            let value = match parts[2].parse::<f64>() {
                Ok(val) => val,
                Err(_) => continue,
            };
            
            let timestamp = parts[3].to_string();
            
            let record = DataRecord {
                id,
                name,
                value,
                timestamp,
            };
            
            self.records.push(record);
            count += 1;
        }
        
        Ok(count)
    }

    pub fn filter_by_value(&self, min_value: f64, max_value: f64) -> Vec<DataRecord> {
        self.records
            .iter()
            .filter(|record| record.value >= min_value && record.value <= max_value)
            .cloned()
            .collect()
    }

    pub fn calculate_statistics(&self) -> (f64, f64, f64) {
        if self.records.is_empty() {
            return (0.0, 0.0, 0.0);
        }
        
        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        let count = self.records.len() as f64;
        let mean = sum / count;
        
        let variance: f64 = self.records
            .iter()
            .map(|r| (r.value - mean).powi(2))
            .sum::<f64>() / count;
        
        let std_dev = variance.sqrt();
        
        (mean, variance, std_dev)
    }

    pub fn find_by_id(&self, target_id: u32) -> Option<&DataRecord> {
        self.records.iter().find(|record| record.id == target_id)
    }

    pub fn total_records(&self) -> usize {
        self.records.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,timestamp").unwrap();
        writeln!(temp_file, "1,test1,10.5,2023-01-01").unwrap();
        writeln!(temp_file, "2,test2,20.3,2023-01-02").unwrap();
        writeln!(temp_file, "3,test3,15.7,2023-01-03").unwrap();
        
        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 3);
        assert_eq!(processor.total_records(), 3);
        
        let stats = processor.calculate_statistics();
        assert!(stats.0 > 0.0);
        
        let filtered = processor.filter_by_value(10.0, 20.0);
        assert_eq!(filtered.len(), 2);
        
        let found = processor.find_by_id(2);
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "test2");
    }
}
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRecord {
    pub id: u64,
    pub timestamp: i64,
    pub value: f64,
    pub category: String,
}

#[derive(Debug)]
pub enum ProcessingError {
    InvalidValue,
    InvalidTimestamp,
    CategoryNotFound,
    SerializationError(String),
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessingError::InvalidValue => write!(f, "Value is outside acceptable range"),
            ProcessingError::InvalidTimestamp => write!(f, "Timestamp is invalid"),
            ProcessingError::CategoryNotFound => write!(f, "Category does not exist"),
            ProcessingError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
        }
    }
}

impl Error for ProcessingError {}

pub struct DataProcessor {
    valid_categories: Vec<String>,
    min_value: f64,
    max_value: f64,
}

impl DataProcessor {
    pub fn new(valid_categories: Vec<String>, min_value: f64, max_value: f64) -> Self {
        DataProcessor {
            valid_categories,
            min_value,
            max_value,
        }
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if !self.valid_categories.contains(&record.category) {
            return Err(ProcessingError::CategoryNotFound);
        }

        if record.value < self.min_value || record.value > self.max_value {
            return Err(ProcessingError::InvalidValue);
        }

        if record.timestamp < 0 {
            return Err(ProcessingError::InvalidTimestamp);
        }

        Ok(())
    }

    pub fn transform_record(&self, record: &DataRecord) -> Result<DataRecord, ProcessingError> {
        self.validate_record(record)?;

        let transformed_value = if record.value > 0.0 {
            record.value.ln()
        } else {
            record.value
        };

        let normalized_category = record.category.to_uppercase();

        Ok(DataRecord {
            id: record.id,
            timestamp: record.timestamp,
            value: transformed_value,
            category: normalized_category,
        })
    }

    pub fn process_batch(&self, records: Vec<DataRecord>) -> Vec<Result<DataRecord, ProcessingError>> {
        records
            .into_iter()
            .map(|record| self.transform_record(&record))
            .collect()
    }

    pub fn serialize_to_json(&self, record: &DataRecord) -> Result<String, ProcessingError> {
        serde_json::to_string(record)
            .map_err(|e| ProcessingError::SerializationError(e.to_string()))
    }

    pub fn deserialize_from_json(json_str: &str) -> Result<DataRecord, ProcessingError> {
        serde_json::from_str(json_str)
            .map_err(|e| ProcessingError::SerializationError(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_success() {
        let processor = DataProcessor::new(
            vec!["temperature".to_string(), "pressure".to_string()],
            0.0,
            100.0,
        );

        let record = DataRecord {
            id: 1,
            timestamp: 1625097600,
            value: 25.5,
            category: "temperature".to_string(),
        };

        assert!(processor.validate_record(&record).is_ok());
    }

    #[test]
    fn test_validation_failure() {
        let processor = DataProcessor::new(
            vec!["temperature".to_string()],
            0.0,
            100.0,
        );

        let record = DataRecord {
            id: 1,
            timestamp: 1625097600,
            value: 150.0,
            category: "temperature".to_string(),
        };

        assert!(processor.validate_record(&record).is_err());
    }

    #[test]
    fn test_transform_record() {
        let processor = DataProcessor::new(
            vec!["temperature".to_string()],
            0.0,
            100.0,
        );

        let record = DataRecord {
            id: 1,
            timestamp: 1625097600,
            value: 25.5,
            category: "temperature".to_string(),
        };

        let transformed = processor.transform_record(&record).unwrap();
        assert_eq!(transformed.category, "TEMPERATURE");
        assert!(transformed.value > 0.0);
    }

    #[test]
    fn test_serialization_deserialization() {
        let processor = DataProcessor::new(vec!["test".to_string()], 0.0, 100.0);

        let record = DataRecord {
            id: 42,
            timestamp: 1625097600,
            value: 50.0,
            category: "test".to_string(),
        };

        let json = processor.serialize_to_json(&record).unwrap();
        let deserialized = DataProcessor::deserialize_from_json(&json).unwrap();

        assert_eq!(record.id, deserialized.id);
        assert_eq!(record.timestamp, deserialized.timestamp);
        assert_eq!(record.value, deserialized.value);
        assert_eq!(record.category, deserialized.category);
    }
}