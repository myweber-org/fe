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

        processor.add_validator("email", |s| s.contains('@') && s.contains('.'));
        processor.add_validator("numeric", |s| s.chars().all(|c| c.is_ascii_digit()));
        
        processor.add_transformer("uppercase", |s| s.to_uppercase());
        processor.add_transformer("trim", |s| s.trim().to_string());
        processor.add_transformer("reverse", |s| s.chars().rev().collect());

        processor
    }

    pub fn add_validator<F>(&mut self, name: &str, validator: F)
    where
        F: Fn(&str) -> bool + 'static,
    {
        self.validators.insert(name.to_string(), Box::new(validator));
    }

    pub fn add_transformer<F>(&mut self, name: &str, transformer: F)
    where
        F: Fn(String) -> String + 'static,
    {
        self.transformers.insert(name.to_string(), Box::new(transformer));
    }

    pub fn validate(&self, data: &str, validator_name: &str) -> Result<bool, String> {
        self.validators
            .get(validator_name)
            .map(|validator| validator(data))
            .ok_or_else(|| format!("Validator '{}' not found", validator_name))
    }

    pub fn transform(&self, data: String, transformer_name: &str) -> Result<String, String> {
        self.transformers
            .get(transformer_name)
            .map(|transformer| transformer(data))
            .ok_or_else(|| format!("Transformer '{}' not found", transformer_name))
    }

    pub fn process_pipeline(&self, data: String, operations: &[(&str, &str)]) -> Result<String, String> {
        let mut result = data;
        
        for (op_type, op_name) in operations {
            match *op_type {
                "validate" => {
                    let is_valid = self.validate(&result, op_name)?;
                    if !is_valid {
                        return Err(format!("Validation '{}' failed for data", op_name));
                    }
                }
                "transform" => {
                    result = self.transform(result, op_name)?;
                }
                _ => return Err(format!("Unknown operation type: {}", op_type)),
            }
        }
        
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_validation() {
        let processor = DataProcessor::new();
        assert!(processor.validate("test@example.com", "email").unwrap());
        assert!(!processor.validate("invalid-email", "email").unwrap());
    }

    #[test]
    fn test_numeric_validation() {
        let processor = DataProcessor::new();
        assert!(processor.validate("12345", "numeric").unwrap());
        assert!(!processor.validate("123abc", "numeric").unwrap());
    }

    #[test]
    fn test_uppercase_transformation() {
        let processor = DataProcessor::new();
        let result = processor.transform("hello".to_string(), "uppercase").unwrap();
        assert_eq!(result, "HELLO");
    }

    #[test]
    fn test_processing_pipeline() {
        let processor = DataProcessor::new();
        let operations = vec![
            ("validate", "email"),
            ("transform", "uppercase"),
            ("transform", "reverse"),
        ];
        
        let result = processor.process_pipeline("test@example.com".to_string(), &operations);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "MOC.ELPMAXE@TSET");
    }
}
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    id: u32,
    values: Vec<f64>,
    metadata: HashMap<String, String>,
}

#[derive(Debug)]
pub enum DataError {
    InvalidId,
    EmptyValues,
    ValueOutOfRange(f64),
    MissingMetadata(String),
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataError::InvalidId => write!(f, "Invalid record ID"),
            DataError::EmptyValues => write!(f, "Record contains no values"),
            DataError::ValueOutOfRange(val) => write!(f, "Value {} is out of acceptable range", val),
            DataError::MissingMetadata(key) => write!(f, "Missing required metadata: {}", key),
        }
    }
}

impl Error for DataError {}

impl DataRecord {
    pub fn new(id: u32, values: Vec<f64>, metadata: HashMap<String, String>) -> Result<Self, DataError> {
        if id == 0 {
            return Err(DataError::InvalidId);
        }
        
        if values.is_empty() {
            return Err(DataError::EmptyValues);
        }
        
        for &value in &values {
            if !value.is_finite() {
                return Err(DataError::ValueOutOfRange(value));
            }
        }
        
        Ok(Self {
            id,
            values,
            metadata,
        })
    }
    
    pub fn validate_metadata(&self, required_keys: &[&str]) -> Result<(), DataError> {
        for key in required_keys {
            if !self.metadata.contains_key(*key) {
                return Err(DataError::MissingMetadata(key.to_string()));
            }
        }
        Ok(())
    }
    
    pub fn calculate_statistics(&self) -> (f64, f64, f64) {
        let count = self.values.len() as f64;
        let sum: f64 = self.values.iter().sum();
        let mean = sum / count;
        
        let variance: f64 = self.values.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / count;
        
        let std_dev = variance.sqrt();
        
        (mean, variance, std_dev)
    }
    
    pub fn transform_values<F>(&mut self, transform_fn: F) 
    where
        F: Fn(f64) -> f64,
    {
        self.values = self.values.iter()
            .map(|&x| transform_fn(x))
            .collect();
    }
    
    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }
    
    pub fn get_id(&self) -> u32 {
        self.id
    }
    
    pub fn get_values(&self) -> &[f64] {
        &self.values
    }
    
    pub fn get_metadata(&self) -> &HashMap<String, String> {
        &self.metadata
    }
}

pub fn process_records(records: &mut [DataRecord]) -> Vec<(u32, f64, f64, f64)> {
    records.iter()
        .filter(|record| record.validate_metadata(&["source", "timestamp"]).is_ok())
        .map(|record| {
            let (mean, variance, std_dev) = record.calculate_statistics();
            (record.id, mean, variance, std_dev)
        })
        .collect()
}

pub fn normalize_values(records: &mut [DataRecord]) {
    for record in records {
        let (mean, _, std_dev) = record.calculate_statistics();
        if std_dev > 0.0 {
            record.transform_values(|x| (x - mean) / std_dev);
        }
    }
}
use csv::{ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
pub struct Record {
    id: u32,
    name: String,
    value: f64,
    active: bool,
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

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let mut reader = ReaderBuilder::new()
            .has_headers(true)
            .from_path(path)?;

        for result in reader.deserialize() {
            let record: Record = result?;
            self.records.push(record);
        }

        Ok(())
    }

    pub fn save_to_csv<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn Error>> {
        let mut writer = WriterBuilder::new()
            .has_headers(true)
            .from_path(path)?;

        for record in &self.records {
            writer.serialize(record)?;
        }

        writer.flush()?;
        Ok(())
    }

    pub fn filter_active(&self) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|record| record.active)
            .collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }

        let sum: f64 = self.records.iter().map(|record| record.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn add_record(&mut self, id: u32, name: String, value: f64, active: bool) {
        self.records.push(Record {
            id,
            name,
            value,
            active,
        });
    }

    pub fn get_record_count(&self) -> usize {
        self.records.len()
    }

    pub fn validate_records(&self) -> Vec<String> {
        let mut errors = Vec::new();

        for (index, record) in self.records.iter().enumerate() {
            if record.name.is_empty() {
                errors.push(format!("Record {}: Name cannot be empty", index));
            }

            if record.value < 0.0 {
                errors.push(format!("Record {}: Value cannot be negative", index));
            }
        }

        errors
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        
        processor.add_record(1, "Test1".to_string(), 10.5, true);
        processor.add_record(2, "Test2".to_string(), 20.0, false);
        processor.add_record(3, "Test3".to_string(), 30.5, true);

        assert_eq!(processor.get_record_count(), 3);
        
        let active_records = processor.filter_active();
        assert_eq!(active_records.len(), 2);

        let average = processor.calculate_average();
        assert!(average.is_some());
        assert!((average.unwrap() - 20.3333).abs() < 0.001);

        let errors = processor.validate_records();
        assert!(errors.is_empty());

        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();
        
        assert!(processor.save_to_csv(path).is_ok());
        
        let mut new_processor = DataProcessor::new();
        assert!(new_processor.load_from_csv(path).is_ok());
        assert_eq!(new_processor.get_record_count(), 3);
    }
}