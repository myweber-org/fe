
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
    DuplicateId(u32),
    CategoryNotFound(String),
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessingError::InvalidValue(val) => write!(f, "Invalid value: {}", val),
            ProcessingError::MissingField(field) => write!(f, "Missing required field: {}", field),
            ProcessingError::DuplicateId(id) => write!(f, "Duplicate record id: {}", id),
            ProcessingError::CategoryNotFound(cat) => write!(f, "Category not found: {}", cat),
        }
    }
}

impl Error for ProcessingError {}

pub struct DataProcessor {
    records: HashMap<u32, DataRecord>,
    categories: Vec<String>,
}

impl DataProcessor {
    pub fn new(valid_categories: Vec<String>) -> Self {
        DataProcessor {
            records: HashMap::new(),
            categories: valid_categories,
        }
    }

    pub fn add_record(&mut self, record: DataRecord) -> Result<(), ProcessingError> {
        if record.value < 0.0 || record.value > 1000.0 {
            return Err(ProcessingError::InvalidValue(record.value));
        }

        if !self.categories.contains(&record.category) {
            return Err(ProcessingError::CategoryNotFound(record.category));
        }

        if self.records.contains_key(&record.id) {
            return Err(ProcessingError::DuplicateId(record.id));
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

    pub fn validate_all(&self) -> Vec<ProcessingError> {
        let mut errors = Vec::new();

        for record in self.records.values() {
            if record.value < 0.0 || record.value > 1000.0 {
                errors.push(ProcessingError::InvalidValue(record.value));
            }

            if !self.categories.contains(&record.category) {
                errors.push(ProcessingError::CategoryNotFound(record.category.clone()));
            }
        }

        errors
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
    fn test_invalid_value() {
        let categories = vec!["A".to_string()];
        let mut processor = DataProcessor::new(categories);
        let record = DataRecord {
            id: 1,
            name: "Test".to_string(),
            value: -10.0,
            category: "A".to_string(),
        };

        assert!(processor.add_record(record).is_err());
    }

    #[test]
    fn test_calculate_average() {
        let categories = vec!["A".to_string()];
        let mut processor = DataProcessor::new(categories);

        let records = vec![
            DataRecord {
                id: 1,
                name: "R1".to_string(),
                value: 50.0,
                category: "A".to_string(),
            },
            DataRecord {
                id: 2,
                name: "R2".to_string(),
                value: 100.0,
                category: "A".to_string(),
            },
        ];

        for record in records {
            processor.add_record(record).unwrap();
        }

        assert_eq!(processor.calculate_average(), 75.0);
    }
}