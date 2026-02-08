
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

    pub fn validate(&self, data: &str, validator_name: &str) -> bool {
        self.validators
            .get(validator_name)
            .map_or(false, |validator| validator(data))
    }

    pub fn transform(&self, data: String, transformer_name: &str) -> Option<String> {
        self.transformers
            .get(transformer_name)
            .map(|transformer| transformer(data))
    }

    pub fn process_pipeline(&self, data: String, operations: &[(&str, &str)]) -> Result<String, String> {
        let mut result = data;

        for (op_type, op_name) in operations {
            match *op_type {
                "validate" => {
                    if !self.validate(&result, op_name) {
                        return Err(format!("Validation failed for '{}'", op_name));
                    }
                }
                "transform" => {
                    result = self.transform(result, op_name)
                        .ok_or_else(|| format!("Unknown transformer '{}'", op_name))?;
                }
                _ => return Err(format!("Unknown operation type '{}'", op_type)),
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
        assert!(processor.validate("test@example.com", "email"));
        assert!(!processor.validate("invalid-email", "email"));
    }

    #[test]
    fn test_numeric_validation() {
        let processor = DataProcessor::new();
        assert!(processor.validate("12345", "numeric"));
        assert!(!processor.validate("12a45", "numeric"));
    }

    #[test]
    fn test_uppercase_transformation() {
        let processor = DataProcessor::new();
        let result = processor.transform("hello".to_string(), "uppercase");
        assert_eq!(result, Some("HELLO".to_string()));
    }

    #[test]
    fn test_processing_pipeline() {
        let processor = DataProcessor::new();
        let operations = vec![
            ("validate", "email"),
            ("transform", "uppercase"),
        ];
        
        let result = processor.process_pipeline("test@example.com".to_string(), &operations);
        assert_eq!(result, Ok("TEST@EXAMPLE.COM".to_string()));
    }
}