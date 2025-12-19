
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