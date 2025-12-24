
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

        for (line_number, line) in reader.lines().enumerate() {
            let line_content = line?;
            
            if line_number == 0 && self.has_header {
                continue;
            }

            let fields: Vec<String> = line_content
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            if !fields.is_empty() && !fields.iter().all(|f| f.is_empty()) {
                records.push(fields);
            }
        }

        Ok(records)
    }

    pub fn validate_record(&self, record: &[String]) -> bool {
        !record.is_empty() && record.iter().all(|field| !field.is_empty())
    }

    pub fn calculate_statistics(&self, records: &[Vec<String>], column_index: usize) -> Option<(f64, f64, f64)> {
        let values: Vec<f64> = records
            .iter()
            .filter_map(|record| record.get(column_index))
            .filter_map(|value| value.parse::<f64>().ok())
            .collect();

        if values.is_empty() {
            return None;
        }

        let sum: f64 = values.iter().sum();
        let count = values.len() as f64;
        let mean = sum / count;

        let variance: f64 = values
            .iter()
            .map(|value| {
                let diff = mean - value;
                diff * diff
            })
            .sum::<f64>()
            / count;

        let std_dev = variance.sqrt();

        Some((mean, variance, std_dev))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_process_file_with_header() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,score").unwrap();
        writeln!(temp_file, "Alice,25,95.5").unwrap();
        writeln!(temp_file, "Bob,30,87.2").unwrap();
        writeln!(temp_file, "Charlie,35,91.8").unwrap();

        let processor = DataProcessor::new(',', true);
        let result = processor.process_file(temp_file.path()).unwrap();

        assert_eq!(result.len(), 3);
        assert_eq!(result[0], vec!["Alice", "25", "95.5"]);
    }

    #[test]
    fn test_validate_record() {
        let processor = DataProcessor::new(',', false);
        
        assert!(processor.validate_record(&["test".to_string(), "123".to_string()]));
        assert!(!processor.validate_record(&[]));
        assert!(!processor.validate_record(&["".to_string(), "data".to_string()]));
    }

    #[test]
    fn test_calculate_statistics() {
        let records = vec![
            vec!["10.5".to_string()],
            vec!["20.3".to_string()],
            vec!["15.7".to_string()],
            vec!["invalid".to_string()],
        ];

        let processor = DataProcessor::new(',', false);
        let stats = processor.calculate_statistics(&records, 0).unwrap();

        let expected_mean = (10.5 + 20.3 + 15.7) / 3.0;
        assert!((stats.0 - expected_mean).abs() < 0.0001);
    }
}
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
            Box::new(|input: &str| input.contains('@') && input.contains('.')),
        );
        
        self.validators.insert(
            "numeric".to_string(),
            Box::new(|input: &str| input.chars().all(|c| c.is_ascii_digit())),
        );
        
        self.validators.insert(
            "alphanumeric".to_string(),
            Box::new(|input: &str| input.chars().all(|c| c.is_ascii_alphanumeric())),
        );
    }
    
    fn register_default_transformers(&mut self) {
        self.transformers.insert(
            "uppercase".to_string(),
            Box::new(|input: String| input.to_uppercase()),
        );
        
        self.transformers.insert(
            "lowercase".to_string(),
            Box::new(|input: String| input.to_lowercase()),
        );
        
        self.transformers.insert(
            "trim".to_string(),
            Box::new(|input: String| input.trim().to_string()),
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
        let trimmed = self.transform("trim", input.to_string());
        
        if trimmed.is_empty() {
            return Err("Input cannot be empty after trimming".to_string());
        }
        
        if !self.validate("alphanumeric", &trimmed) {
            return Err("Input contains invalid characters".to_string());
        }
        
        let processed = self.transform("uppercase", trimmed);
        Ok(processed)
    }
    
    pub fn register_validator<F>(&mut self, name: String, validator: F)
    where
        F: Fn(&str) -> bool + 'static,
    {
        self.validators.insert(name, Box::new(validator));
    }
    
    pub fn register_transformer<F>(&mut self, name: String, transformer: F)
    where
        F: Fn(String) -> String + 'static,
    {
        self.transformers.insert(name, Box::new(transformer));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_email_validation() {
        let processor = DataProcessor::new();
        assert!(processor.validate("email", "test@example.com"));
        assert!(!processor.validate("email", "invalid-email"));
    }
    
    #[test]
    fn test_numeric_validation() {
        let processor = DataProcessor::new();
        assert!(processor.validate("numeric", "12345"));
        assert!(!processor.validate("numeric", "123abc"));
    }
    
    #[test]
    fn test_data_processing() {
        let processor = DataProcessor::new();
        let result = processor.process_data("  hello123  ");
        assert_eq!(result, Ok("HELLO123".to_string()));
        
        let invalid_result = processor.process_data("  hello!@#  ");
        assert!(invalid_result.is_err());
    }
    
    #[test]
    fn test_custom_validator() {
        let mut processor = DataProcessor::new();
        processor.register_validator("even_length".to_string(), |input: &str| input.len() % 2 == 0);
        
        assert!(processor.validate("even_length", "abcd"));
        assert!(!processor.validate("even_length", "abc"));
    }
}
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProcessingError {
    #[error("Invalid data format")]
    InvalidFormat,
    #[error("Missing required field: {0}")]
    MissingField(String),
    #[error("Value out of range: {0}")]
    OutOfRange(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRecord {
    pub id: u64,
    pub timestamp: i64,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

impl DataRecord {
    pub fn validate(&self) -> Result<(), ProcessingError> {
        if self.id == 0 {
            return Err(ProcessingError::InvalidFormat);
        }
        
        if self.timestamp < 0 {
            return Err(ProcessingError::OutOfRange("timestamp".to_string()));
        }
        
        if self.values.is_empty() {
            return Err(ProcessingError::MissingField("values".to_string()));
        }
        
        Ok(())
    }
    
    pub fn normalize_values(&mut self) {
        if let Some(max) = self.values.iter().copied().reduce(f64::max) {
            if max != 0.0 {
                for value in &mut self.values {
                    *value /= max;
                }
            }
        }
    }
    
    pub fn calculate_statistics(&self) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        
        if !self.values.is_empty() {
            let sum: f64 = self.values.iter().sum();
            let count = self.values.len() as f64;
            let mean = sum / count;
            
            let variance: f64 = self.values
                .iter()
                .map(|&v| (v - mean).powi(2))
                .sum::<f64>() / count;
            
            stats.insert("mean".to_string(), mean);
            stats.insert("variance".to_string(), variance);
            stats.insert("min".to_string(), self.values.iter().copied().reduce(f64::min).unwrap());
            stats.insert("max".to_string(), self.values.iter().copied().reduce(f64::max).unwrap());
            stats.insert("count".to_string(), count);
        }
        
        stats
    }
}

pub fn process_records(records: &mut [DataRecord]) -> Result<Vec<HashMap<String, f64>>, ProcessingError> {
    let mut results = Vec::new();
    
    for record in records {
        record.validate()?;
        record.normalize_values();
        results.push(record.calculate_statistics());
    }
    
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_record_validation() {
        let valid_record = DataRecord {
            id: 1,
            timestamp: 1000,
            values: vec![1.0, 2.0, 3.0],
            metadata: HashMap::new(),
        };
        
        assert!(valid_record.validate().is_ok());
        
        let invalid_record = DataRecord {
            id: 0,
            timestamp: 1000,
            values: vec![1.0, 2.0, 3.0],
            metadata: HashMap::new(),
        };
        
        assert!(invalid_record.validate().is_err());
    }
    
    #[test]
    fn test_normalize_values() {
        let mut record = DataRecord {
            id: 1,
            timestamp: 1000,
            values: vec![1.0, 2.0, 3.0],
            metadata: HashMap::new(),
        };
        
        record.normalize_values();
        assert_eq!(record.values, vec![1.0/3.0, 2.0/3.0, 1.0]);
    }
    
    #[test]
    fn test_calculate_statistics() {
        let record = DataRecord {
            id: 1,
            timestamp: 1000,
            values: vec![1.0, 2.0, 3.0],
            metadata: HashMap::new(),
        };
        
        let stats = record.calculate_statistics();
        assert_eq!(stats.get("mean"), Some(&2.0));
        assert_eq!(stats.get("count"), Some(&3.0));
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub timestamp: String,
    pub value: f64,
    pub category: String,
}

impl DataRecord {
    pub fn new(id: u32, timestamp: String, value: f64, category: String) -> Self {
        DataRecord {
            id,
            timestamp,
            value,
            category,
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.timestamp.is_empty() && self.value >= 0.0 && !self.category.is_empty()
    }
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

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<usize, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut count = 0;

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            
            if line_num == 0 {
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

            let timestamp = parts[1].to_string();
            let value = match parts[2].parse::<f64>() {
                Ok(val) => val,
                Err(_) => continue,
            };

            let category = parts[3].to_string();

            let record = DataRecord::new(id, timestamp, value, category);
            if record.is_valid() {
                self.records.push(record);
                count += 1;
            }
        }

        Ok(count)
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }

        let sum: f64 = self.records.iter().map(|record| record.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn get_statistics(&self) -> (f64, f64, f64) {
        if self.records.is_empty() {
            return (0.0, 0.0, 0.0);
        }

        let values: Vec<f64> = self.records.iter().map(|record| record.value).collect();
        let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let avg = self.calculate_average().unwrap_or(0.0);

        (min, max, avg)
    }

    pub fn count_records(&self) -> usize {
        self.records.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_record_validation() {
        let valid_record = DataRecord::new(1, "2023-01-01".to_string(), 42.5, "A".to_string());
        assert!(valid_record.is_valid());

        let invalid_record = DataRecord::new(2, "".to_string(), -1.0, "".to_string());
        assert!(!invalid_record.is_valid());
    }

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        assert_eq!(processor.count_records(), 0);

        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,timestamp,value,category").unwrap();
        writeln!(temp_file, "1,2023-01-01,10.5,A").unwrap();
        writeln!(temp_file, "2,2023-01-02,20.0,B").unwrap();
        writeln!(temp_file, "3,2023-01-03,15.5,A").unwrap();

        let count = processor.load_from_csv(temp_file.path()).unwrap();
        assert_eq!(count, 3);
        assert_eq!(processor.count_records(), 3);

        let category_a = processor.filter_by_category("A");
        assert_eq!(category_a.len(), 2);

        let avg = processor.calculate_average().unwrap();
        assert!((avg - 15.333).abs() < 0.001);

        let (min, max, avg_stat) = processor.get_statistics();
        assert_eq!(min, 10.5);
        assert_eq!(max, 20.0);
        assert!((avg_stat - 15.333).abs() < 0.001);
    }
}