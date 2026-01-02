
use std::collections::HashMap;

pub struct DataProcessor {
    validators: HashMap<String, Box<dyn Fn(&str) -> bool>>,
    transformers: HashMap<String, Box<dyn Fn(String) -> String>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        let mut processor = DataProcessor {
            validators: HashMap::new(),
            transformers: HashMap::new(),
        };
        
        processor.register_default_validators();
        processor.register_default_transformers();
        
        processor
    }
    
    fn register_default_validators(&mut self) {
        self.validators.insert(
            "email".to_string(),
            Box::new(|input: &str| {
                input.contains('@') && input.contains('.') && input.len() > 5
            })
        );
        
        self.validators.insert(
            "numeric".to_string(),
            Box::new(|input: &str| {
                input.parse::<f64>().is_ok()
            })
        );
    }
    
    fn register_default_transformers(&mut self) {
        self.transformers.insert(
            "uppercase".to_string(),
            Box::new(|input: String| {
                input.to_uppercase()
            })
        );
        
        self.transformers.insert(
            "trim".to_string(),
            Box::new(|input: String| {
                input.trim().to_string()
            })
        );
    }
    
    pub fn validate(&self, validator_name: &str, input: &str) -> bool {
        match self.validators.get(validator_name) {
            Some(validator) => validator(input),
            None => false,
        }
    }
    
    pub fn transform(&self, transformer_name: &str, input: String) -> String {
        match self.transformers.get(transformer_name) {
            Some(transformer) => transformer(input),
            None => input,
        }
    }
    
    pub fn process_data(&self, input: &str) -> Result<String, String> {
        if !self.validate("email", input) {
            return Err("Invalid email format".to_string());
        }
        
        let trimmed = self.transform("trim", input.to_string());
        let processed = self.transform("uppercase", trimmed);
        
        Ok(processed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_email_validation() {
        let processor = DataProcessor::new();
        assert!(processor.validate("email", "test@example.com"));
        assert!(!processor.validate("email", "invalid"));
    }
    
    #[test]
    fn test_numeric_validation() {
        let processor = DataProcessor::new();
        assert!(processor.validate("numeric", "123.45"));
        assert!(!processor.validate("numeric", "abc"));
    }
    
    #[test]
    fn test_uppercase_transformation() {
        let processor = DataProcessor::new();
        let result = processor.transform("uppercase", "hello".to_string());
        assert_eq!(result, "HELLO");
    }
    
    #[test]
    fn test_trim_transformation() {
        let processor = DataProcessor::new();
        let result = processor.transform("trim", "  hello  ".to_string());
        assert_eq!(result, "hello");
    }
    
    #[test]
    fn test_process_data() {
        let processor = DataProcessor::new();
        let result = processor.process_data("  user@domain.com  ");
        assert_eq!(result.unwrap(), "USER@DOMAIN.COM");
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

pub fn process_data_file(file_path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let mut reader = Reader::from_reader(file);
    let mut records = Vec::new();

    for result in reader.deserialize() {
        let record: Record = result?;
        if record.value >= 0.0 {
            records.push(record);
        }
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
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct DataRecord {
    id: u32,
    value: f64,
    timestamp: String,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, timestamp: String) -> Result<Self, String> {
        if value < 0.0 {
            return Err("Value cannot be negative".to_string());
        }
        if timestamp.is_empty() {
            return Err("Timestamp cannot be empty".to_string());
        }
        Ok(Self {
            id,
            value,
            timestamp,
        })
    }
}

pub fn process_csv_file<P: AsRef<Path>>(path: P) -> Result<Vec<DataRecord>, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();
    let mut line_number = 0;

    for line in reader.lines() {
        line_number += 1;
        let line = line?;
        
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }

        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() != 3 {
            return Err(format!("Invalid CSV format at line {}", line_number).into());
        }

        let id = parts[0].parse::<u32>()
            .map_err(|_| format!("Invalid ID at line {}", line_number))?;
        
        let value = parts[1].parse::<f64>()
            .map_err(|_| format!("Invalid value at line {}", line_number))?;
        
        let timestamp = parts[2].trim().to_string();

        match DataRecord::new(id, value, timestamp) {
            Ok(record) => records.push(record),
            Err(e) => return Err(format!("Error at line {}: {}", line_number, e).into()),
        }
    }

    if records.is_empty() {
        return Err("No valid records found in file".into());
    }

    Ok(records)
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
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_valid_record_creation() {
        let record = DataRecord::new(1, 42.5, "2024-01-15T10:30:00Z".to_string());
        assert!(record.is_ok());
        let record = record.unwrap();
        assert_eq!(record.id, 1);
        assert_eq!(record.value, 42.5);
        assert_eq!(record.timestamp, "2024-01-15T10:30:00Z");
    }

    #[test]
    fn test_invalid_record_creation() {
        let record = DataRecord::new(1, -5.0, "2024-01-15T10:30:00Z".to_string());
        assert!(record.is_err());
        
        let record = DataRecord::new(1, 5.0, "".to_string());
        assert!(record.is_err());
    }

    #[test]
    fn test_process_csv_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "1,42.5,2024-01-15T10:30:00Z").unwrap();
        writeln!(temp_file, "2,18.3,2024-01-15T11:00:00Z").unwrap();
        writeln!(temp_file, "# This is a comment").unwrap();
        writeln!(temp_file, "").unwrap();
        writeln!(temp_file, "3,99.9,2024-01-15T12:00:00Z").unwrap();

        let records = process_csv_file(temp_file.path()).unwrap();
        assert_eq!(records.len(), 3);
        assert_eq!(records[0].id, 1);
        assert_eq!(records[1].value, 18.3);
        assert_eq!(records[2].timestamp, "2024-01-15T12:00:00Z");
    }

    #[test]
    fn test_calculate_statistics() {
        let records = vec![
            DataRecord::new(1, 10.0, "2024-01-15T10:30:00Z".to_string()).unwrap(),
            DataRecord::new(2, 20.0, "2024-01-15T11:00:00Z".to_string()).unwrap(),
            DataRecord::new(3, 30.0, "2024-01-15T12:00:00Z".to_string()).unwrap(),
        ];

        let (mean, variance, std_dev) = calculate_statistics(&records);
        assert_eq!(mean, 20.0);
        assert_eq!(variance, 66.66666666666667);
        assert_eq!(std_dev, 8.16496580927726);
    }

    #[test]
    fn test_empty_statistics() {
        let records: Vec<DataRecord> = vec![];
        let (mean, variance, std_dev) = calculate_statistics(&records);
        assert_eq!(mean, 0.0);
        assert_eq!(variance, 0.0);
        assert_eq!(std_dev, 0.0);
    }
}