
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
        self.validators
            .get(name)
            .map_or(false, |validator| validator(data))
    }

    pub fn transform(&self, name: &str, data: String) -> Option<String> {
        self.transformers
            .get(name)
            .map(|transformer| transformer(data))
    }

    pub fn process_pipeline(&self, data: &str, validators: &[&str], transformers: &[&str]) -> Option<String> {
        for validator_name in validators {
            if !self.validate(validator_name, data) {
                return None;
            }
        }

        let mut result = data.to_string();
        for transformer_name in transformers {
            if let Some(transformed) = self.transform(transformer_name, result) {
                result = transformed;
            } else {
                return None;
            }
        }

        Some(result)
    }
}

pub fn create_default_processor() -> DataProcessor {
    let mut processor = DataProcessor::new();

    processor.register_validator("not_empty", Box::new(|s| !s.trim().is_empty()));
    processor.register_validator("is_numeric", Box::new(|s| s.chars().all(|c| c.is_ascii_digit())));

    processor.register_transformer("uppercase", Box::new(|s| s.to_uppercase()));
    processor.register_transformer("trim", Box::new(|s| s.trim().to_string()));
    processor.register_transformer("reverse", Box::new(|s| s.chars().rev().collect()));

    processor
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation() {
        let processor = create_default_processor();
        assert!(processor.validate("not_empty", "hello"));
        assert!(!processor.validate("not_empty", "   "));
        assert!(processor.validate("is_numeric", "12345"));
        assert!(!processor.validate("is_numeric", "123a"));
    }

    #[test]
    fn test_transformation() {
        let processor = create_default_processor();
        assert_eq!(processor.transform("uppercase", "hello".to_string()), Some("HELLO".to_string()));
        assert_eq!(processor.transform("trim", "  hello  ".to_string()), Some("hello".to_string()));
        assert_eq!(processor.transform("reverse", "hello".to_string()), Some("olleh".to_string()));
    }

    #[test]
    fn test_pipeline() {
        let processor = create_default_processor();
        let result = processor.process_pipeline("  hello  ", &["not_empty"], &["trim", "uppercase"]);
        assert_eq!(result, Some("HELLO".to_string()));

        let invalid_result = processor.process_pipeline("   ", &["not_empty"], &["trim"]);
        assert_eq!(invalid_result, None);
    }
}