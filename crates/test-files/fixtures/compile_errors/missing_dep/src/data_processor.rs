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