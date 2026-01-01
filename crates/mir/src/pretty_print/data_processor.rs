
use csv::{Reader, Writer};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    active: bool,
}

impl Record {
    fn is_valid(&self) -> bool {
        !self.name.is_empty() && self.value >= 0.0
    }
}

pub fn process_csv(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(Path::new(input_path))?;
    let mut reader = Reader::from_reader(input_file);
    
    let output_file = File::create(Path::new(output_path))?;
    let mut writer = Writer::from_writer(output_file);
    
    let mut valid_count = 0;
    let mut invalid_count = 0;
    
    for result in reader.deserialize() {
        let record: Record = result?;
        
        if record.is_valid() {
            writer.serialize(&record)?;
            valid_count += 1;
        } else {
            invalid_count += 1;
        }
    }
    
    writer.flush()?;
    
    println!("Processing complete:");
    println!("  Valid records: {}", valid_count);
    println!("  Invalid records: {}", invalid_count);
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_valid_record() {
        let record = Record {
            id: 1,
            name: "Test".to_string(),
            value: 42.5,
            active: true,
        };
        assert!(record.is_valid());
    }
    
    #[test]
    fn test_invalid_record() {
        let record = Record {
            id: 2,
            name: "".to_string(),
            value: -10.0,
            active: false,
        };
        assert!(!record.is_valid());
    }
}
use std::collections::HashMap;

pub struct DataProcessor {
    validators: HashMap<String, Box<dyn Fn(&str) -> bool>>,
    transformers: HashMap<String, Box<dyn Fn(String) -> String>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            validators: HashMap::new(),
            transformers: HashMap::new(),
        }
    }

    pub fn register_validator(&mut self, name: &str, validator: Box<dyn Fn(&str) -> bool>) {
        self.validators.insert(name.to_string(), validator);
    }

    pub fn register_transformer(&mut self, name: &str, transformer: Box<dyn Fn(String) -> String>) {
        self.transformers.insert(name.to_string(), transformer);
    }

    pub fn validate(&self, name: &str, data: &str) -> bool {
        match self.validators.get(name) {
            Some(validator) => validator(data),
            None => false,
        }
    }

    pub fn transform(&self, name: &str, data: String) -> Option<String> {
        self.transformers.get(name).map(|transformer| transformer(data))
    }

    pub fn process_pipeline(&self, data: String, validators: &[&str], transformers: &[&str]) -> Option<String> {
        for validator_name in validators {
            if !self.validate(validator_name, &data) {
                return None;
            }
        }

        let mut result = data;
        for transformer_name in transformers {
            match self.transform(transformer_name, result) {
                Some(transformed) => result = transformed,
                None => return None,
            }
        }

        Some(result)
    }
}

pub fn create_default_processor() -> DataProcessor {
    let mut processor = DataProcessor::new();

    processor.register_validator("is_numeric", Box::new(|s| s.parse::<f64>().is_ok()));
    processor.register_validator("is_alpha", Box::new(|s| s.chars().all(|c| c.is_alphabetic())));

    processor.register_transformer("to_uppercase", Box::new(|s| s.to_uppercase()));
    processor.register_transformer("trim_spaces", Box::new(|s| s.trim().to_string()));
    processor.register_transformer("reverse_string", Box::new(|s| s.chars().rev().collect()));

    processor
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation() {
        let processor = create_default_processor();
        assert!(processor.validate("is_numeric", "123.45"));
        assert!(!processor.validate("is_numeric", "abc"));
    }

    #[test]
    fn test_transformation() {
        let processor = create_default_processor();
        let result = processor.transform("to_uppercase", "hello".to_string());
        assert_eq!(result, Some("HELLO".to_string()));
    }

    #[test]
    fn test_pipeline() {
        let processor = create_default_processor();
        let result = processor.process_pipeline(
            "  test  ".to_string(),
            &["is_alpha"],
            &["trim_spaces", "to_uppercase"]
        );
        assert_eq!(result, Some("TEST".to_string()));
    }
}