
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    id: u32,
    name: String,
    value: f64,
    metadata: HashMap<String, String>,
}

#[derive(Debug)]
pub enum ValidationError {
    InvalidId,
    EmptyName,
    NegativeValue,
    MissingMetadata(String),
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::InvalidId => write!(f, "ID must be greater than zero"),
            ValidationError::EmptyName => write!(f, "Name cannot be empty"),
            ValidationError::NegativeValue => write!(f, "Value cannot be negative"),
            ValidationError::MissingMetadata(key) => write!(f, "Missing metadata key: {}", key),
        }
    }
}

impl Error for ValidationError {}

impl DataRecord {
    pub fn new(id: u32, name: String, value: f64) -> Self {
        Self {
            id,
            name,
            value,
            metadata: HashMap::new(),
        }
    }

    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.id == 0 {
            return Err(ValidationError::InvalidId);
        }
        if self.name.trim().is_empty() {
            return Err(ValidationError::EmptyName);
        }
        if self.value < 0.0 {
            return Err(ValidationError::NegativeValue);
        }
        Ok(())
    }

    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }

    pub fn transform_value<F>(&mut self, transformer: F)
    where
        F: Fn(f64) -> f64,
    {
        self.value = transformer(self.value);
    }

    pub fn calculate_score(&self) -> f64 {
        let base_score = self.value * 100.0;
        let metadata_bonus = self.metadata.len() as f64 * 5.0;
        base_score + metadata_bonus
    }
}

pub fn process_records(records: &mut [DataRecord]) -> Result<Vec<f64>, ValidationError> {
    let mut scores = Vec::with_capacity(records.len());
    
    for record in records.iter_mut() {
        record.validate()?;
        record.transform_value(|v| v * 1.1);
        scores.push(record.calculate_score());
    }
    
    Ok(scores)
}

pub fn filter_records(records: &[DataRecord], min_value: f64) -> Vec<&DataRecord> {
    records
        .iter()
        .filter(|r| r.value >= min_value)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_record() {
        let record = DataRecord::new(1, "Test".to_string(), 42.5);
        assert!(record.validate().is_ok());
    }

    #[test]
    fn test_invalid_id() {
        let record = DataRecord::new(0, "Test".to_string(), 42.5);
        assert!(matches!(record.validate(), Err(ValidationError::InvalidId)));
    }

    #[test]
    fn test_calculate_score() {
        let mut record = DataRecord::new(1, "Test".to_string(), 10.0);
        record.add_metadata("category".to_string(), "premium".to_string());
        let score = record.calculate_score();
        assert_eq!(score, 1005.0);
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
            Box::new(|s: &str| s.contains('@') && s.contains('.')),
        );
        
        self.validators.insert(
            "numeric".to_string(),
            Box::new(|s: &str| s.parse::<f64>().is_ok()),
        );
        
        self.validators.insert(
            "not_empty".to_string(),
            Box::new(|s: &str| !s.trim().is_empty()),
        );
    }
    
    fn register_default_transformers(&mut self) {
        self.transformers.insert(
            "uppercase".to_string(),
            Box::new(|s: String| s.to_uppercase()),
        );
        
        self.transformers.insert(
            "trim".to_string(),
            Box::new(|s: String| s.trim().to_string()),
        );
        
        self.transformers.insert(
            "reverse".to_string(),
            Box::new(|s: String| s.chars().rev().collect()),
        );
    }
    
    pub fn validate(&self, validator_name: &str, data: &str) -> bool {
        self.validators
            .get(validator_name)
            .map(|validator| validator(data))
            .unwrap_or(false)
    }
    
    pub fn transform(&self, transformer_name: &str, data: String) -> String {
        self.transformers
            .get(transformer_name)
            .map(|transformer| transformer(data))
            .unwrap_or(data)
    }
    
    pub fn process_pipeline(&self, data: &str, operations: Vec<(&str, &str)>) -> Result<String, String> {
        let mut result = data.to_string();
        
        for (op_type, op_name) in operations {
            match op_type {
                "validate" => {
                    if !self.validate(op_name, &result) {
                        return Err(format!("Validation '{}' failed for data: {}", op_name, result));
                    }
                }
                "transform" => {
                    result = self.transform(op_name, result);
                }
                _ => return Err(format!("Unknown operation type: {}", op_type)),
            }
        }
        
        Ok(result)
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
        assert!(processor.validate("numeric", "123.45"));
        assert!(!processor.validate("numeric", "abc"));
    }
    
    #[test]
    fn test_transformation_pipeline() {
        let processor = DataProcessor::new();
        let result = processor.process_pipeline(
            "  hello  ",
            vec![
                ("validate", "not_empty"),
                ("transform", "trim"),
                ("transform", "uppercase"),
            ],
        );
        
        assert_eq!(result.unwrap(), "HELLO");
    }
    
    #[test]
    fn test_custom_validator() {
        let mut processor = DataProcessor::new();
        processor.register_validator("even_length", |s: &str| s.len() % 2 == 0);
        
        assert!(processor.validate("even_length", "ab"));
        assert!(!processor.validate("even_length", "abc"));
    }
}
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub category: String,
}

#[derive(Debug)]
pub enum ProcessingError {
    InvalidValue(f64),
    MissingField(String),
    CategoryNotFound(String),
    DuplicateId(u32),
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessingError::InvalidValue(val) => write!(f, "Invalid value: {}", val),
            ProcessingError::MissingField(field) => write!(f, "Missing required field: {}", field),
            ProcessingError::CategoryNotFound(cat) => write!(f, "Category not found: {}", cat),
            ProcessingError::DuplicateId(id) => write!(f, "Duplicate record ID: {}", id),
        }
    }
}

impl Error for ProcessingError {}

pub struct DataProcessor {
    records: HashMap<u32, DataRecord>,
    categories: Vec<String>,
}

impl DataProcessor {
    pub fn new(categories: Vec<String>) -> Self {
        DataProcessor {
            records: HashMap::new(),
            categories,
        }
    }

    pub fn add_record(&mut self, record: DataRecord) -> Result<(), ProcessingError> {
        if self.records.contains_key(&record.id) {
            return Err(ProcessingError::DuplicateId(record.id));
        }

        if !self.categories.contains(&record.category) {
            return Err(ProcessingError::CategoryNotFound(record.category.clone()));
        }

        if record.value < 0.0 || record.value > 1000.0 {
            return Err(ProcessingError::InvalidValue(record.value));
        }

        if record.name.trim().is_empty() {
            return Err(ProcessingError::MissingField("name".to_string()));
        }

        self.records.insert(record.id, record);
        Ok(())
    }

    pub fn get_record(&self, id: u32) -> Option<&DataRecord> {
        self.records.get(&id)
    }

    pub fn calculate_average(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }

        let sum: f64 = self.records.values().map(|r| r.value).sum();
        sum / self.records.len() as f64
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .values()
            .filter(|r| r.category == category)
            .collect()
    }

    pub fn transform_values<F>(&mut self, transform_fn: F)
    where
        F: Fn(f64) -> f64,
    {
        for record in self.records.values_mut() {
            record.value = transform_fn(record.value);
        }
    }

    pub fn get_statistics(&self) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        
        if self.records.is_empty() {
            return stats;
        }

        let values: Vec<f64> = self.records.values().map(|r| r.value).collect();
        let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let avg = self.calculate_average();

        stats.insert("min".to_string(), min);
        stats.insert("max".to_string(), max);
        stats.insert("average".to_string(), avg);
        stats.insert("count".to_string(), self.records.len() as f64);

        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_valid_record() {
        let categories = vec!["A".to_string(), "B".to_string()];
        let mut processor = DataProcessor::new(categories);
        
        let record = DataRecord {
            id: 1,
            name: "Test".to_string(),
            value: 100.0,
            category: "A".to_string(),
        };

        assert!(processor.add_record(record).is_ok());
        assert_eq!(processor.records.len(), 1);
    }

    #[test]
    fn test_duplicate_id() {
        let categories = vec!["A".to_string()];
        let mut processor = DataProcessor::new(categories);
        
        let record1 = DataRecord {
            id: 1,
            name: "First".to_string(),
            value: 50.0,
            category: "A".to_string(),
        };

        let record2 = DataRecord {
            id: 1,
            name: "Second".to_string(),
            value: 60.0,
            category: "A".to_string(),
        };

        assert!(processor.add_record(record1).is_ok());
        assert!(processor.add_record(record2).is_err());
    }

    #[test]
    fn test_calculate_average() {
        let categories = vec!["A".to_string()];
        let mut processor = DataProcessor::new(categories);
        
        let records = vec![
            DataRecord { id: 1, name: "R1".to_string(), value: 10.0, category: "A".to_string() },
            DataRecord { id: 2, name: "R2".to_string(), value: 20.0, category: "A".to_string() },
            DataRecord { id: 3, name: "R3".to_string(), value: 30.0, category: "A".to_string() },
        ];

        for record in records {
            processor.add_record(record).unwrap();
        }

        assert_eq!(processor.calculate_average(), 20.0);
    }
}