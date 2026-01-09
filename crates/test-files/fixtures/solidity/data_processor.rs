
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub tags: Vec<String>,
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
    validation_rules: HashMap<String, Box<dyn Fn(&DataRecord) -> Result<(), ProcessingError>>>,
    transformation_pipeline: Vec<Box<dyn Fn(DataRecord) -> Result<DataRecord, ProcessingError>>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            validation_rules: HashMap::new(),
            transformation_pipeline: Vec::new(),
        }
    }

    pub fn add_validation_rule(
        &mut self,
        name: &str,
        rule: Box<dyn Fn(&DataRecord) -> Result<(), ProcessingError>>,
    ) {
        self.validation_rules.insert(name.to_string(), rule);
    }

    pub fn add_transformation(
        &mut self,
        transform: Box<dyn Fn(DataRecord) -> Result<DataRecord, ProcessingError>>,
    ) {
        self.transformation_pipeline.push(transform);
    }

    pub fn process_record(&self, mut record: DataRecord) -> Result<DataRecord, ProcessingError> {
        for (rule_name, rule) in &self.validation_rules {
            rule(&record).map_err(|e| {
                ProcessingError::ValidationError(format!("Rule '{}' failed: {}", rule_name, e))
            })?;
        }

        for (index, transform) in self.transformation_pipeline.iter().enumerate() {
            record = transform(record).map_err(|e| {
                ProcessingError::TransformationFailed(format!("Step {} failed: {}", index + 1, e))
            })?;
        }

        Ok(record)
    }

    pub fn process_batch(
        &self,
        records: Vec<DataRecord>,
    ) -> (Vec<DataRecord>, Vec<ProcessingError>) {
        let mut processed = Vec::new();
        let mut errors = Vec::new();

        for record in records {
            match self.process_record(record) {
                Ok(processed_record) => processed.push(processed_record),
                Err(e) => errors.push(e),
            }
        }

        (processed, errors)
    }
}

fn validate_positive_value(record: &DataRecord) -> Result<(), ProcessingError> {
    if record.value < 0.0 {
        Err(ProcessingError::ValidationError(
            "Value must be positive".to_string(),
        ))
    } else {
        Ok(())
    }
}

fn normalize_name(record: DataRecord) -> Result<DataRecord, ProcessingError> {
    let normalized_name = record.name.trim().to_lowercase();
    if normalized_name.is_empty() {
        Err(ProcessingError::TransformationFailed(
            "Name cannot be empty after normalization".to_string(),
        ))
    } else {
        Ok(DataRecord {
            name: normalized_name,
            ..record
        })
    }
}

fn add_default_tag(record: DataRecord) -> Result<DataRecord, ProcessingError> {
    let mut new_tags = record.tags;
    if !new_tags.contains(&"processed".to_string()) {
        new_tags.push("processed".to_string());
    }
    Ok(DataRecord {
        tags: new_tags,
        ..record
    })
}

pub fn create_default_processor() -> DataProcessor {
    let mut processor = DataProcessor::new();
    
    processor.add_validation_rule(
        "positive_value",
        Box::new(validate_positive_value),
    );
    
    processor.add_transformation(Box::new(normalize_name));
    processor.add_transformation(Box::new(add_default_tag));
    
    processor
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_positive_validation() {
        let record = DataRecord {
            id: 1,
            name: "test".to_string(),
            value: -5.0,
            tags: vec![],
        };

        let result = validate_positive_value(&record);
        assert!(result.is_err());
    }

    #[test]
    fn test_normalize_name() {
        let record = DataRecord {
            id: 1,
            name: "  TEST  ".to_string(),
            value: 10.0,
            tags: vec![],
        };

        let result = normalize_name(record).unwrap();
        assert_eq!(result.name, "test");
    }

    #[test]
    fn test_full_processing() {
        let processor = create_default_processor();
        
        let record = DataRecord {
            id: 1,
            name: "  Sample Data  ".to_string(),
            value: 42.5,
            tags: vec!["original".to_string()],
        };

        let result = processor.process_record(record).unwrap();
        assert_eq!(result.name, "sample data");
        assert!(result.tags.contains(&"processed".to_string()));
        assert!(result.tags.contains(&"original".to_string()));
    }
}