
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug)]
pub enum ProcessingError {
    InvalidData(String),
    TransformationFailed(String),
    ValidationError(String),
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessingError::InvalidData(msg) => write!(f, "Invalid data: {}", msg),
            ProcessingError::TransformationFailed(msg) => write!(f, "Transformation failed: {}", msg),
            ProcessingError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl Error for ProcessingError {}

pub struct DataProcessor {
    validation_rules: Vec<Box<dyn Fn(&DataRecord) -> Result<(), ProcessingError>>>,
    transformation_pipeline: Vec<Box<dyn Fn(DataRecord) -> Result<DataRecord, ProcessingError>>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            validation_rules: Vec::new(),
            transformation_pipeline: Vec::new(),
        }
    }

    pub fn add_validation_rule<F>(&mut self, rule: F)
    where
        F: Fn(&DataRecord) -> Result<(), ProcessingError> + 'static,
    {
        self.validation_rules.push(Box::new(rule));
    }

    pub fn add_transformation<F>(&mut self, transform: F)
    where
        F: Fn(DataRecord) -> Result<DataRecord, ProcessingError> + 'static,
    {
        self.transformation_pipeline.push(Box::new(transform));
    }

    pub fn process_record(&self, mut record: DataRecord) -> Result<DataRecord, ProcessingError> {
        for rule in &self.validation_rules {
            rule(&record)?;
        }

        for transform in &self.transformation_pipeline {
            record = transform(record)?;
        }

        Ok(record)
    }

    pub fn process_batch(&self, records: Vec<DataRecord>) -> Result<Vec<DataRecord>, ProcessingError> {
        let mut results = Vec::with_capacity(records.len());
        
        for record in records {
            match self.process_record(record) {
                Ok(processed) => results.push(processed),
                Err(e) => return Err(e),
            }
        }
        
        Ok(results)
    }
}

pub fn create_default_processor() -> DataProcessor {
    let mut processor = DataProcessor::new();
    
    processor.add_validation_rule(|record| {
        if record.name.is_empty() {
            Err(ProcessingError::ValidationError("Name cannot be empty".to_string()))
        } else {
            Ok(())
        }
    });
    
    processor.add_validation_rule(|record| {
        if record.value < 0.0 {
            Err(ProcessingError::ValidationError("Value must be non-negative".to_string()))
        } else {
            Ok(())
        }
    });
    
    processor.add_transformation(|mut record| {
        record.name = record.name.to_uppercase();
        Ok(record)
    });
    
    processor.add_transformation(|mut record| {
        record.value = (record.value * 100.0).round() / 100.0;
        Ok(record)
    });
    
    processor
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_data_processing() {
        let processor = create_default_processor();
        
        let mut metadata = HashMap::new();
        metadata.insert("category".to_string(), "test".to_string());
        
        let record = DataRecord {
            id: 1,
            name: "test record".to_string(),
            value: 123.456,
            metadata,
        };
        
        let result = processor.process_record(record);
        assert!(result.is_ok());
        
        let processed = result.unwrap();
        assert_eq!(processed.name, "TEST RECORD");
        assert_eq!(processed.value, 123.46);
    }
    
    #[test]
    fn test_validation_failure() {
        let processor = create_default_processor();
        
        let record = DataRecord {
            id: 2,
            name: "".to_string(),
            value: 50.0,
            metadata: HashMap::new(),
        };
        
        let result = processor.process_record(record);
        assert!(result.is_err());
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

    processor.register_validator("not_empty", Box::new(|s: &str| !s.trim().is_empty()));
    processor.register_validator("is_numeric", Box::new(|s: &str| s.chars().all(|c| c.is_ascii_digit())));

    processor.register_transformer("trim", Box::new(|s: String| s.trim().to_string()));
    processor.register_transformer("uppercase", Box::new(|s: String| s.to_uppercase()));
    processor.register_transformer("reverse", Box::new(|s: String| s.chars().rev().collect()));

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
        assert!(!processor.validate("is_numeric", "12a45"));
    }

    #[test]
    fn test_transformation() {
        let processor = create_default_processor();
        assert_eq!(processor.transform("trim", "  hello  ".to_string()), Some("hello".to_string()));
        assert_eq!(processor.transform("uppercase", "hello".to_string()), Some("HELLO".to_string()));
        assert_eq!(processor.transform("reverse", "hello".to_string()), Some("olleh".to_string()));
    }

    #[test]
    fn test_pipeline() {
        let processor = create_default_processor();
        let result = processor.process_pipeline(
            "  hello  ".to_string(),
            &["not_empty"],
            &["trim", "uppercase"]
        );
        assert_eq!(result, Some("HELLO".to_string()));
    }
}