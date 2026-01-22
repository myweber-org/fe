
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
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct ValidationError {
    message: String,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Validation error: {}", self.message)
    }
}

impl Error for ValidationError {}

pub struct DataProcessor {
    data: HashMap<String, Vec<f64>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            data: HashMap::new(),
        }
    }

    pub fn add_dataset(&mut self, key: &str, values: Vec<f64>) -> Result<(), ValidationError> {
        if values.is_empty() {
            return Err(ValidationError {
                message: format!("Dataset '{}' cannot be empty", key),
            });
        }

        if values.iter().any(|&v| v.is_nan() || v.is_infinite()) {
            return Err(ValidationError {
                message: format!("Dataset '{}' contains invalid numeric values", key),
            });
        }

        self.data.insert(key.to_string(), values);
        Ok(())
    }

    pub fn calculate_statistics(&self, key: &str) -> Result<Statistics, ValidationError> {
        let values = self.data.get(key).ok_or_else(|| ValidationError {
            message: format!("Dataset '{}' not found", key),
        })?;

        let count = values.len();
        let sum: f64 = values.iter().sum();
        let mean = sum / count as f64;
        
        let variance: f64 = values.iter()
            .map(|&v| (v - mean).powi(2))
            .sum::<f64>() / count as f64;
        
        let std_dev = variance.sqrt();

        Ok(Statistics {
            count,
            sum,
            mean,
            variance,
            std_dev,
        })
    }

    pub fn normalize_data(&self, key: &str) -> Result<Vec<f64>, ValidationError> {
        let values = self.data.get(key).ok_or_else(|| ValidationError {
            message: format!("Dataset '{}' not found", key),
        })?;

        let stats = self.calculate_statistics(key)?;
        
        if stats.std_dev == 0.0 {
            return Ok(vec![0.0; values.len()]);
        }

        let normalized: Vec<f64> = values.iter()
            .map(|&v| (v - stats.mean) / stats.std_dev)
            .collect();

        Ok(normalized)
    }

    pub fn merge_datasets(&mut self, key1: &str, key2: &str, new_key: &str) -> Result<(), ValidationError> {
        let values1 = self.data.get(key1).ok_or_else(|| ValidationError {
            message: format!("Dataset '{}' not found", key1),
        })?;

        let values2 = self.data.get(key2).ok_or_else(|| ValidationError {
            message: format!("Dataset '{}' not found", key2),
        })?;

        let mut merged = values1.clone();
        merged.extend_from_slice(values2);

        self.add_dataset(new_key, merged)
    }

    pub fn list_datasets(&self) -> Vec<String> {
        self.data.keys().cloned().collect()
    }
}

pub struct Statistics {
    pub count: usize,
    pub sum: f64,
    pub mean: f64,
    pub variance: f64,
    pub std_dev: f64,
}

impl fmt::Display for Statistics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Count: {}, Sum: {:.4}, Mean: {:.4}, Variance: {:.4}, Std Dev: {:.4}",
            self.count, self.sum, self.mean, self.variance, self.std_dev
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_dataset() {
        let mut processor = DataProcessor::new();
        let result = processor.add_dataset("test", vec![1.0, 2.0, 3.0]);
        assert!(result.is_ok());
        assert_eq!(processor.list_datasets(), vec!["test"]);
    }

    #[test]
    fn test_empty_dataset() {
        let mut processor = DataProcessor::new();
        let result = processor.add_dataset("empty", vec![]);
        assert!(result.is_err());
    }

    #[test]
    fn test_calculate_statistics() {
        let mut processor = DataProcessor::new();
        processor.add_dataset("numbers", vec![1.0, 2.0, 3.0, 4.0, 5.0]).unwrap();
        
        let stats = processor.calculate_statistics("numbers").unwrap();
        assert_eq!(stats.count, 5);
        assert_eq!(stats.sum, 15.0);
        assert_eq!(stats.mean, 3.0);
    }

    #[test]
    fn test_normalize_data() {
        let mut processor = DataProcessor::new();
        processor.add_dataset("values", vec![1.0, 2.0, 3.0]).unwrap();
        
        let normalized = processor.normalize_data("values").unwrap();
        assert_eq!(normalized.len(), 3);
    }
}