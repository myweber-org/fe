
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
    transformation_functions: Vec<Box<dyn Fn(DataRecord) -> Result<DataRecord, ProcessingError>>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            validation_rules: Vec::new(),
            transformation_functions: Vec::new(),
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
        self.transformation_functions.push(Box::new(transform));
    }

    pub fn process_record(&self, mut record: DataRecord) -> Result<DataRecord, ProcessingError> {
        for rule in &self.validation_rules {
            rule(&record)?;
        }

        for transform in &self.transformation_functions {
            record = transform(record)?;
        }

        Ok(record)
    }

    pub fn process_batch(&self, records: Vec<DataRecord>) -> Result<Vec<DataRecord>, ProcessingError> {
        let mut processed_records = Vec::with_capacity(records.len());
        
        for record in records {
            match self.process_record(record) {
                Ok(processed) => processed_records.push(processed),
                Err(e) => return Err(e),
            }
        }
        
        Ok(processed_records)
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
            Err(ProcessingError::ValidationError("Value cannot be negative".to_string()))
        } else {
            Ok(())
        }
    });
    
    processor.add_transformation(|mut record| {
        record.name = record.name.trim().to_string();
        if record.name.is_empty() {
            Err(ProcessingError::TransformationFailed("Name became empty after trimming".to_string()))
        } else {
            Ok(record)
        }
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
        metadata.insert("source".to_string(), "test".to_string());
        
        let record = DataRecord {
            id: 1,
            name: "  Test Record  ".to_string(),
            value: 123.456789,
            metadata,
        };
        
        let result = processor.process_record(record);
        assert!(result.is_ok());
        
        let processed = result.unwrap();
        assert_eq!(processed.name, "Test Record");
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