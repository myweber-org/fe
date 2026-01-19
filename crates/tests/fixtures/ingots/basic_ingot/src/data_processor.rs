
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub timestamp: i64,
}

#[derive(Debug)]
pub enum ProcessingError {
    InvalidValue,
    InvalidTimestamp,
    SerializationError(String),
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessingError::InvalidValue => write!(f, "Invalid data value"),
            ProcessingError::InvalidTimestamp => write!(f, "Invalid timestamp"),
            ProcessingError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
        }
    }
}

impl Error for ProcessingError {}

pub struct DataProcessor {
    threshold: f64,
}

impl DataProcessor {
    pub fn new(threshold: f64) -> Self {
        DataProcessor { threshold }
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if record.value.is_nan() || record.value.is_infinite() {
            return Err(ProcessingError::InvalidValue);
        }

        if record.timestamp < 0 {
            return Err(ProcessingError::InvalidTimestamp);
        }

        Ok(())
    }

    pub fn process_record(&self, record: &DataRecord) -> Result<DataRecord, ProcessingError> {
        self.validate_record(record)?;

        let processed_value = if record.value > self.threshold {
            record.value * 0.9
        } else {
            record.value * 1.1
        };

        Ok(DataRecord {
            id: record.id,
            value: processed_value,
            timestamp: record.timestamp,
        })
    }

    pub fn serialize_record(&self, record: &DataRecord) -> Result<String, ProcessingError> {
        serde_json::to_string(record)
            .map_err(|e| ProcessingError::SerializationError(e.to_string()))
    }

    pub fn deserialize_record(&self, data: &str) -> Result<DataRecord, ProcessingError> {
        serde_json::from_str(data)
            .map_err(|e| ProcessingError::SerializationError(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_valid_record() {
        let processor = DataProcessor::new(100.0);
        let record = DataRecord {
            id: 1,
            value: 50.0,
            timestamp: 1234567890,
        };
        assert!(processor.validate_record(&record).is_ok());
    }

    #[test]
    fn test_validation_invalid_value() {
        let processor = DataProcessor::new(100.0);
        let record = DataRecord {
            id: 1,
            value: f64::NAN,
            timestamp: 1234567890,
        };
        assert!(processor.validate_record(&record).is_err());
    }

    #[test]
    fn test_processing_above_threshold() {
        let processor = DataProcessor::new(100.0);
        let record = DataRecord {
            id: 1,
            value: 150.0,
            timestamp: 1234567890,
        };
        let processed = processor.process_record(&record).unwrap();
        assert_eq!(processed.value, 135.0);
    }

    #[test]
    fn test_serialization_roundtrip() {
        let processor = DataProcessor::new(100.0);
        let original = DataRecord {
            id: 42,
            value: 75.5,
            timestamp: 987654321,
        };
        
        let serialized = processor.serialize_record(&original).unwrap();
        let deserialized = processor.deserialize_record(&serialized).unwrap();
        
        assert_eq!(original.id, deserialized.id);
        assert_eq!(original.value, deserialized.value);
        assert_eq!(original.timestamp, deserialized.timestamp);
    }
}use csv::Reader;
use serde::Deserialize;
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

pub fn process_csv_file(path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(path)?;
    let mut reader = Reader::from_reader(file);
    let mut records = Vec::new();

    for result in reader.deserialize() {
        let record: Record = result?;
        if record.value < 0.0 {
            return Err(format!("Invalid value in record ID {}", record.id).into());
        }
        records.push(record);
    }

    Ok(records)
}

pub fn calculate_statistics(records: &[Record]) -> (f64, f64, f64) {
    let count = records.len() as f64;
    if count == 0.0 {
        return (0.0, 0.0, 0.0);
    }

    let sum: f64 = records.iter().map(|r| r.value).sum();
    let mean = sum / count;
    
    let variance: f64 = records.iter()
        .map(|r| (r.value - mean).powi(2))
        .sum::<f64>() / count;
    
    let std_dev = variance.sqrt();
    
    (sum, mean, std_dev)
}

pub fn filter_by_category(records: Vec<Record>, category: &str) -> Vec<Record> {
    records.into_iter()
        .filter(|r| r.category == category)
        .collect()
}use csv::Reader;
use serde::Deserialize;
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

pub fn process_data_file(file_path: &str, category_filter: Option<&str>) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let mut reader = Reader::from_reader(file);
    let mut filtered_records = Vec::new();

    for result in reader.deserialize() {
        let record: Record = result?;
        
        match category_filter {
            Some(filter) if record.category == filter => filtered_records.push(record),
            None => filtered_records.push(record),
            _ => continue,
        }
    }

    Ok(filtered_records)
}

pub fn calculate_statistics(records: &[Record]) -> (f64, f64, f64) {
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
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_process_data_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,category").unwrap();
        writeln!(temp_file, "1,ItemA,10.5,Category1").unwrap();
        writeln!(temp_file, "2,ItemB,20.3,Category2").unwrap();
        
        let records = process_data_file(temp_file.path().to_str().unwrap(), None).unwrap();
        assert_eq!(records.len(), 2);
    }

    #[test]
    fn test_calculate_statistics() {
        let records = vec![
            Record { id: 1, name: "Test1".to_string(), value: 10.0, category: "Cat1".to_string() },
            Record { id: 2, name: "Test2".to_string(), value: 20.0, category: "Cat1".to_string() },
        ];
        
        let (mean, variance, std_dev) = calculate_statistics(&records);
        assert_eq!(mean, 15.0);
        assert_eq!(variance, 25.0);
        assert_eq!(std_dev, 5.0);
    }
}