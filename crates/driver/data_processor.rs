
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;

pub struct DataProcessor {
    data: Vec<f64>,
    metadata: HashMap<String, String>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            data: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn load_from_csv(&mut self, filepath: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(filepath)?;
        let reader = BufReader::new(file);
        
        self.data.clear();
        
        for (index, line) in reader.lines().enumerate() {
            let line = line?;
            
            if index == 0 {
                self.parse_header(&line);
                continue;
            }
            
            if let Ok(value) = line.trim().parse::<f64>() {
                self.data.push(value);
            }
        }
        
        self.metadata.insert("source".to_string(), filepath.to_string());
        self.metadata.insert("loaded_timestamp".to_string(), chrono::Local::now().to_rfc3339());
        
        Ok(())
    }
    
    fn parse_header(&mut self, header_line: &str) {
        let parts: Vec<&str> = header_line.split(',').collect();
        if parts.len() >= 2 {
            self.metadata.insert("column_name".to_string(), parts[0].to_string());
            self.metadata.insert("data_type".to_string(), parts[1].to_string());
        }
    }
    
    pub fn calculate_statistics(&self) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        
        if self.data.is_empty() {
            return stats;
        }
        
        let count = self.data.len() as f64;
        let sum: f64 = self.data.iter().sum();
        let mean = sum / count;
        
        let variance: f64 = self.data.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / count;
        
        let std_dev = variance.sqrt();
        
        let min = self.data.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = self.data.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        
        stats.insert("count".to_string(), count);
        stats.insert("sum".to_string(), sum);
        stats.insert("mean".to_string(), mean);
        stats.insert("variance".to_string(), variance);
        stats.insert("std_dev".to_string(), std_dev);
        stats.insert("min".to_string(), min);
        stats.insert("max".to_string(), max);
        
        stats
    }
    
    pub fn filter_data(&self, threshold: f64) -> Vec<f64> {
        self.data.iter()
            .filter(|&&x| x >= threshold)
            .cloned()
            .collect()
    }
    
    pub fn get_metadata(&self) -> &HashMap<String, String> {
        &self.metadata
    }
    
    pub fn data_summary(&self) -> String {
        format!(
            "Data points: {}, Source: {}",
            self.data.len(),
            self.metadata.get("source").unwrap_or(&"Unknown".to_string())
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_data_processing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "value,f64").unwrap();
        writeln!(temp_file, "10.5").unwrap();
        writeln!(temp_file, "20.3").unwrap();
        writeln!(temp_file, "15.7").unwrap();
        
        let filepath = temp_file.path().to_str().unwrap();
        
        let mut processor = DataProcessor::new();
        let result = processor.load_from_csv(filepath);
        
        assert!(result.is_ok());
        assert_eq!(processor.data.len(), 3);
        
        let stats = processor.calculate_statistics();
        assert_eq!(stats["count"], 3.0);
        assert_eq!(stats["min"], 10.5);
        assert_eq!(stats["max"], 20.3);
        
        let filtered = processor.filter_data(15.0);
        assert_eq!(filtered.len(), 2);
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
            Box::new(|input: &str| {
                input.contains('@') && input.contains('.') && input.len() > 5
            }),
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

    pub fn process_pipeline(&self, input: String, operations: Vec<(&str, &str)>) -> Result<String, String> {
        let mut result = input;

        for (op_type, op_name) in operations {
            match op_type {
                "validate" => {
                    if !self.validate(op_name, &result) {
                        return Err(format!("Validation failed for '{}'", op_name));
                    }
                }
                "transform" => {
                    result = self.transform(op_name, result);
                }
                _ => return Err(format!("Unknown operation type: {}", op_type)),
            }
        }

        Ok(result)
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
    fn test_transformation_pipeline() {
        let processor = DataProcessor::new();
        let result = processor.process_pipeline(
            "  Hello World  ".to_string(),
            vec![("transform", "trim"), ("transform", "uppercase")],
        );
        assert_eq!(result.unwrap(), "HELLO WORLD");
    }

    #[test]
    fn test_custom_validator() {
        let mut processor = DataProcessor::new();
        processor.register_validator("even_length", |s: &str| s.len() % 2 == 0);
        
        assert!(processor.validate("even_length", "ab"));
        assert!(!processor.validate("even_length", "abc"));
    }
}
use std::collections::HashMap;

pub struct DataProcessor {
    validators: Vec<Box<dyn Fn(&str) -> bool>>,
    transformers: HashMap<String, Box<dyn Fn(String) -> String>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            validators: Vec::new(),
            transformers: HashMap::new(),
        }
    }

    pub fn add_validator<F>(&mut self, validator: F)
    where
        F: Fn(&str) -> bool + 'static,
    {
        self.validators.push(Box::new(validator));
    }

    pub fn add_transformer<F>(&mut self, name: &str, transformer: F)
    where
        F: Fn(String) -> String + 'static,
    {
        self.transformers
            .insert(name.to_string(), Box::new(transformer));
    }

    pub fn validate(&self, input: &str) -> bool {
        self.validators.iter().all(|v| v(input))
    }

    pub fn transform(&self, name: &str, input: String) -> Option<String> {
        self.transformers.get(name).map(|t| t(input))
    }

    pub fn process_pipeline(&self, input: &str, transformations: &[&str]) -> Option<String> {
        if !self.validate(input) {
            return None;
        }

        let mut result = input.to_string();
        for &transform_name in transformations {
            if let Some(transformed) = self.transform(transform_name, result) {
                result = transformed;
            } else {
                return None;
            }
        }
        Some(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation() {
        let mut processor = DataProcessor::new();
        processor.add_validator(|s| !s.is_empty());
        processor.add_validator(|s| s.len() <= 10);

        assert!(processor.validate("test"));
        assert!(!processor.validate(""));
        assert!(!processor.validate("verylongstring"));
    }

    #[test]
    fn test_transformation() {
        let mut processor = DataProcessor::new();
        processor.add_transformer("uppercase", |s| s.to_uppercase());
        processor.add_transformer("reverse", |s| s.chars().rev().collect());

        assert_eq!(
            processor.transform("uppercase", "hello".to_string()),
            Some("HELLO".to_string())
        );
        assert_eq!(
            processor.transform("reverse", "rust".to_string()),
            Some("tsur".to_string())
        );
    }

    #[test]
    fn test_pipeline() {
        let mut processor = DataProcessor::new();
        processor.add_validator(|s| s.len() > 0);
        processor.add_transformer("uppercase", |s| s.to_uppercase());
        processor.add_transformer("add_exclamation", |s| format!("{}!", s));

        let result = processor.process_pipeline("hello", &["uppercase", "add_exclamation"]);
        assert_eq!(result, Some("HELLO!".to_string()));

        let invalid_result = processor.process_pipeline("", &["uppercase"]);
        assert_eq!(invalid_result, None);
    }
}