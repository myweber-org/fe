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

struct DataProcessor {
    records: Vec<Record>,
}

impl DataProcessor {
    fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let mut rdr = Reader::from_reader(file);
        
        for result in rdr.deserialize() {
            let record: Record = result?;
            self.records.push(record);
        }
        
        Ok(())
    }

    fn filter_by_category(&self, category: &str) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    fn calculate_average(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }
        
        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        sum / self.records.len() as f64
    }

    fn find_max_value(&self) -> Option<&Record> {
        self.records.iter().max_by(|a, b| {
            a.value.partial_cmp(&b.value).unwrap()
        })
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut processor = DataProcessor::new();
    
    match processor.load_from_csv("data.csv") {
        Ok(_) => {
            println!("Loaded {} records", processor.records.len());
            
            let filtered = processor.filter_by_category("premium");
            println!("Premium records: {}", filtered.len());
            
            let avg = processor.calculate_average();
            println!("Average value: {:.2}", avg);
            
            if let Some(max_record) = processor.find_max_value() {
                println!("Max value record: {:?}", max_record);
            }
        }
        Err(e) => eprintln!("Error loading CSV: {}", e),
    }
    
    Ok(())
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

        processor.add_validator("email", |s| s.contains('@') && s.contains('.'));
        processor.add_validator("phone", |s| s.chars().all(|c| c.is_numeric()));
        
        processor.add_transformer("uppercase", |s| s.to_uppercase());
        processor.add_transformer("trim", |s| s.trim().to_string());

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

    pub fn validate(&self, name: &str, data: &str) -> bool {
        self.validators
            .get(name)
            .map_or(false, |validator| validator(data))
    }

    pub fn transform(&self, name: &str, data: String) -> Option<String> {
        self.transformers
            .get(name)
            .map(|transformer| transformer(data))
    }

    pub fn process_pipeline(&self, data: &str, operations: &[(&str, &str)]) -> Result<String, String> {
        let mut result = data.to_string();

        for (op_type, op_name) in operations {
            match *op_type {
                "validate" => {
                    if !self.validate(op_name, &result) {
                        return Err(format!("Validation failed for '{}'", op_name));
                    }
                }
                "transform" => {
                    result = self.transform(op_name, result)
                        .ok_or_else(|| format!("Transformer '{}' not found", op_name))?;
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
        assert!(processor.validate("email", "test@example.com"));
        assert!(!processor.validate("email", "invalid-email"));
    }

    #[test]
    fn test_phone_validation() {
        let processor = DataProcessor::new();
        assert!(processor.validate("phone", "1234567890"));
        assert!(!processor.validate("phone", "123-456-7890"));
    }

    #[test]
    fn test_uppercase_transformation() {
        let processor = DataProcessor::new();
        let result = processor.transform("uppercase", "hello".to_string());
        assert_eq!(result, Some("HELLO".to_string()));
    }

    #[test]
    fn test_processing_pipeline() {
        let processor = DataProcessor::new();
        let operations = vec![
            ("validate", "email"),
            ("transform", "uppercase"),
            ("transform", "trim"),
        ];
        
        let result = processor.process_pipeline("  test@example.com  ", &operations);
        assert_eq!(result, Ok("TEST@EXAMPLE.COM".to_string()));
    }
}