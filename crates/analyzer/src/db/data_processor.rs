
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

    pub fn process_pipeline(&self, data: String, operations: &[(&str, &str)]) -> Result<String, String> {
        let mut current = data;

        for (op_type, op_name) in operations {
            match *op_type {
                "validate" => {
                    if !self.validate(op_name, &current) {
                        return Err(format!("Validation failed for operation: {}", op_name));
                    }
                }
                "transform" => {
                    match self.transform(op_name, current) {
                        Some(transformed) => current = transformed,
                        None => return Err(format!("Transformation not found: {}", op_name)),
                    }
                }
                _ => return Err(format!("Unknown operation type: {}", op_type)),
            }
        }

        Ok(current)
    }
}

pub fn create_default_processor() -> DataProcessor {
    let mut processor = DataProcessor::new();

    processor.register_validator("non_empty", Box::new(|s| !s.trim().is_empty()));
    processor.register_validator("is_numeric", Box::new(|s| s.chars().all(|c| c.is_digit(10))));

    processor.register_transformer("to_uppercase", Box::new(|s| s.to_uppercase()));
    processor.register_transformer("trim_spaces", Box::new(|s| s.trim().to_string()));
    processor.register_transformer("reverse", Box::new(|s| s.chars().rev().collect()));

    processor
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation() {
        let processor = create_default_processor();
        assert!(processor.validate("non_empty", "hello"));
        assert!(!processor.validate("non_empty", ""));
        assert!(processor.validate("is_numeric", "123"));
        assert!(!processor.validate("is_numeric", "abc"));
    }

    #[test]
    fn test_transformation() {
        let processor = create_default_processor();
        assert_eq!(processor.transform("to_uppercase", "hello".to_string()), Some("HELLO".to_string()));
        assert_eq!(processor.transform("trim_spaces", "  hello  ".to_string()), Some("hello".to_string()));
        assert_eq!(processor.transform("reverse", "abc".to_string()), Some("cba".to_string()));
    }

    #[test]
    fn test_pipeline() {
        let processor = create_default_processor();
        let operations = [
            ("validate", "non_empty"),
            ("transform", "to_uppercase"),
            ("transform", "reverse"),
        ];

        let result = processor.process_pipeline("hello".to_string(), &operations);
        assert_eq!(result, Ok("OLLEH".to_string()));

        let invalid_result = processor.process_pipeline("".to_string(), &operations);
        assert!(invalid_result.is_err());
    }
}