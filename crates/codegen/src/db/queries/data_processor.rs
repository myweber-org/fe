
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

        processor.register_validator("email", |s| s.contains('@') && s.contains('.'));
        processor.register_validator("numeric", |s| s.chars().all(|c| c.is_ascii_digit()));
        
        processor.register_transformer("uppercase", |s| s.to_uppercase());
        processor.register_transformer("trim", |s| s.trim().to_string());

        processor
    }

    pub fn register_validator<F>(&mut self, name: &str, validator: F)
    where
        F: Fn(&str) -> bool + 'static,
    {
        self.validators.insert(name.to_string(), Box::new(validator));
    }

    pub fn register_transformer<F>(&mut self, name: &str, transformer: F)
    where
        F: Fn(String) -> String + 'static,
    {
        self.transformers.insert(name.to_string(), Box::new(transformer));
    }

    pub fn validate(&self, data: &str, validator_name: &str) -> Result<bool, String> {
        match self.validators.get(validator_name) {
            Some(validator) => Ok(validator(data)),
            None => Err(format!("Validator '{}' not found", validator_name)),
        }
    }

    pub fn transform(&self, data: String, transformer_name: &str) -> Result<String, String> {
        match self.transformers.get(transformer_name) {
            Some(transformer) => Ok(transformer(data)),
            None => Err(format!("Transformer '{}' not found", transformer_name)),
        }
    }

    pub fn process_pipeline(&self, data: String, operations: Vec<(&str, &str)>) -> Result<String, String> {
        let mut result = data;
        
        for (op_type, op_name) in operations {
            match op_type {
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
            ("validate", "numeric"),
            ("transform", "uppercase"),
        ];
        
        let result = processor.process_pipeline("12345".to_string(), operations);
        assert_eq!(result.unwrap(), "12345");
        
        let invalid_result = processor.process_pipeline("abc".to_string(), operations);
        assert!(invalid_result.is_err());
    }
}
use csv::Reader;
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

    (mean, variance, std_dev)
}

pub fn filter_by_category(records: Vec<Record>, category: &str) -> Vec<Record> {
    records.into_iter()
        .filter(|r| r.category == category)
        .collect()
}