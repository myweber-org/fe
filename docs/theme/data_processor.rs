
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct DataProcessor {
    delimiter: char,
    has_header: bool,
}

impl DataProcessor {
    pub fn new(delimiter: char, has_header: bool) -> Self {
        DataProcessor {
            delimiter,
            has_header,
        }
    }

    pub fn process_file<P: AsRef<Path>>(&self, file_path: P) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        
        let mut records = Vec::new();
        let mut lines = reader.lines();
        
        if self.has_header {
            lines.next();
        }
        
        for line_result in lines {
            let line = line_result?;
            let fields: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();
            
            if !fields.is_empty() && !fields.iter().all(|f| f.is_empty()) {
                records.push(fields);
            }
        }
        
        Ok(records)
    }
    
    pub fn validate_records(&self, records: &[Vec<String>]) -> Result<(), String> {
        if records.is_empty() {
            return Err("No records found".to_string());
        }
        
        let expected_len = records[0].len();
        for (i, record) in records.iter().enumerate() {
            if record.len() != expected_len {
                return Err(format!("Record {} has {} fields, expected {}", 
                    i + 1, record.len(), expected_len));
            }
            
            for (j, field) in record.iter().enumerate() {
                if field.is_empty() {
                    return Err(format!("Empty field at record {}, position {}", 
                        i + 1, j + 1));
                }
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_process_csv() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "John,30,New York").unwrap();
        writeln!(temp_file, "Alice,25,London").unwrap();
        
        let processor = DataProcessor::new(',', true);
        let result = processor.process_file(temp_file.path());
        
        assert!(result.is_ok());
        let records = result.unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0], vec!["John", "30", "New York"]);
    }
    
    #[test]
    fn test_validation() {
        let records = vec![
            vec!["a".to_string(), "b".to_string(), "c".to_string()],
            vec!["d".to_string(), "e".to_string(), "f".to_string()],
        ];
        
        let processor = DataProcessor::new(',', false);
        let result = processor.validate_records(&records);
        assert!(result.is_ok());
    }
}
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DataRecord {
    pub id: u64,
    pub timestamp: i64,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug)]
pub enum ValidationError {
    InvalidId,
    InvalidTimestamp,
    EmptyValues,
    MissingRequiredMetadata,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::InvalidId => write!(f, "ID must be greater than zero"),
            ValidationError::InvalidTimestamp => write!(f, "Timestamp cannot be negative"),
            ValidationError::EmptyValues => write!(f, "Values array cannot be empty"),
            ValidationError::MissingRequiredMetadata => write!(f, "Required metadata fields missing"),
        }
    }
}

impl Error for ValidationError {}

pub fn validate_record(record: &DataRecord) -> Result<(), ValidationError> {
    if record.id == 0 {
        return Err(ValidationError::InvalidId);
    }
    
    if record.timestamp < 0 {
        return Err(ValidationError::InvalidTimestamp);
    }
    
    if record.values.is_empty() {
        return Err(ValidationError::EmptyValues);
    }
    
    let required_fields = ["source", "version"];
    for field in required_fields.iter() {
        if !record.metadata.contains_key(*field) {
            return Err(ValidationError::MissingRequiredMetadata);
        }
    }
    
    Ok(())
}

pub fn normalize_values(record: &mut DataRecord) {
    if record.values.is_empty() {
        return;
    }
    
    let sum: f64 = record.values.iter().sum();
    let mean = sum / record.values.len() as f64;
    
    let variance: f64 = record.values.iter()
        .map(|&x| (x - mean).powi(2))
        .sum::<f64>() / record.values.len() as f64;
    
    let std_dev = variance.sqrt();
    
    if std_dev > 0.0 {
        for value in record.values.iter_mut() {
            *value = (*value - mean) / std_dev;
        }
    }
}

pub fn calculate_statistics(record: &DataRecord) -> HashMap<String, f64> {
    let mut stats = HashMap::new();
    
    if record.values.is_empty() {
        return stats;
    }
    
    let count = record.values.len() as f64;
    let sum: f64 = record.values.iter().sum();
    let mean = sum / count;
    
    let min = record.values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
    let max = record.values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
    
    let variance: f64 = record.values.iter()
        .map(|&x| (x - mean).powi(2))
        .sum::<f64>() / count;
    
    stats.insert("count".to_string(), count);
    stats.insert("sum".to_string(), sum);
    stats.insert("mean".to_string(), mean);
    stats.insert("min".to_string(), min);
    stats.insert("max".to_string(), max);
    stats.insert("variance".to_string(), variance);
    stats.insert("std_dev".to_string(), variance.sqrt());
    
    stats
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_validate_record_valid() {
        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), "sensor_01".to_string());
        metadata.insert("version".to_string(), "1.0".to_string());
        
        let record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: vec![1.0, 2.0, 3.0],
            metadata,
        };
        
        assert!(validate_record(&record).is_ok());
    }
    
    #[test]
    fn test_normalize_values() {
        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), "test".to_string());
        metadata.insert("version".to_string(), "1.0".to_string());
        
        let mut record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: vec![1.0, 2.0, 3.0],
            metadata,
        };
        
        normalize_values(&mut record);
        
        let mean: f64 = record.values.iter().sum::<f64>() / record.values.len() as f64;
        let variance: f64 = record.values.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / record.values.len() as f64;
        
        assert!(mean.abs() < 1e-10);
        assert!((variance - 1.0).abs() < 1e-10);
    }
}use csv::Reader;
use serde::Deserialize;
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize)]
struct Record {
    id: u32,
    value: f64,
    category: String,
}

pub struct DataProcessor {
    records: Vec<Record>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let mut rdr = Reader::from_reader(file);
        
        for result in rdr.deserialize() {
            let record: Record = result?;
            self.records.push(record);
        }
        
        Ok(())
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

    pub fn filter_by_category(&self, category: &str) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|r| r.category == category)
            .collect()
    }

    pub fn get_record_count(&self) -> usize {
        self.records.len()
    }

    pub fn get_max_value(&self) -> Option<f64> {
        self.records.iter().map(|r| r.value).max_by(|a, b| a.partial_cmp(b).unwrap())
    }

    pub fn get_min_value(&self) -> Option<f64> {
        self.records.iter().map(|r| r.value).min_by(|a, b| a.partial_cmp(b).unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processing() {
        let mut processor = DataProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category").unwrap();
        writeln!(temp_file, "1,10.5,category_a").unwrap();
        writeln!(temp_file, "2,20.3,category_b").unwrap();
        writeln!(temp_file, "3,15.7,category_a").unwrap();
        
        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(processor.get_record_count(), 3);
        
        let stats = processor.calculate_statistics();
        assert!((stats.0 - 15.5).abs() < 0.1);
        
        let filtered = processor.filter_by_category("category_a");
        assert_eq!(filtered.len(), 2);
        
        assert_eq!(processor.get_max_value(), Some(20.3));
        assert_eq!(processor.get_min_value(), Some(10.5));
    }
}