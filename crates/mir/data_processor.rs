
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRecord {
    pub id: u64,
    pub timestamp: i64,
    pub values: HashMap<String, f64>,
    pub metadata: Option<HashMap<String, String>>,
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

    pub fn add_transformation<F>(&mut self, transformation: F)
    where
        F: Fn(DataRecord) -> Result<DataRecord, ProcessingError> + 'static,
    {
        self.transformation_pipeline.push(Box::new(transformation));
    }

    pub fn process(&self, mut record: DataRecord) -> Result<DataRecord, ProcessingError> {
        for rule in &self.validation_rules {
            rule(&record)?;
        }

        for transformation in &self.transformation_pipeline {
            record = transformation(record)?;
        }

        Ok(record)
    }

    pub fn batch_process(&self, records: Vec<DataRecord>) -> Vec<Result<DataRecord, ProcessingError>> {
        records.into_iter().map(|record| self.process(record)).collect()
    }
}

pub fn create_default_processor() -> DataProcessor {
    let mut processor = DataProcessor::new();

    processor.add_validation_rule(|record| {
        if record.id == 0 {
            Err(ProcessingError::ValidationError("ID cannot be zero".to_string()))
        } else {
            Ok(())
        }
    });

    processor.add_validation_rule(|record| {
        if record.timestamp < 0 {
            Err(ProcessingError::ValidationError("Timestamp cannot be negative".to_string()))
        } else {
            Ok(())
        }
    });

    processor.add_transformation(|mut record| {
        let normalized_values: HashMap<String, f64> = record.values
            .iter()
            .map(|(key, value)| (key.to_lowercase(), value.abs()))
            .collect();
        
        record.values = normalized_values;
        Ok(record)
    });

    processor.add_transformation(|mut record| {
        if let Some(metadata) = &mut record.metadata {
            metadata.insert("processed".to_string(), "true".to_string());
            metadata.insert("processor_version".to_string(), "1.0.0".to_string());
        }
        Ok(record)
    });

    processor
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_data_processing() {
        let processor = create_default_processor();
        
        let mut values = HashMap::new();
        values.insert("TEMPERATURE".to_string(), -25.5);
        values.insert("Pressure".to_string(), 101.3);
        
        let record = DataRecord {
            id: 1,
            timestamp: 1625097600,
            values,
            metadata: Some(HashMap::new()),
        };

        let result = processor.process(record);
        assert!(result.is_ok());
        
        let processed = result.unwrap();
        assert!(processed.values.contains_key("temperature"));
        assert_eq!(processed.values.get("temperature"), Some(&25.5));
        
        if let Some(metadata) = processed.metadata {
            assert_eq!(metadata.get("processed"), Some(&"true".to_string()));
        }
    }

    #[test]
    fn test_validation_failure() {
        let processor = create_default_processor();
        
        let record = DataRecord {
            id: 0,
            timestamp: 1625097600,
            values: HashMap::new(),
            metadata: None,
        };

        let result = processor.process(record);
        assert!(result.is_err());
    }
}