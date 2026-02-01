
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
pub enum ValidationError {
    InvalidId,
    EmptyName,
    NegativeValue,
    InvalidCategory,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::InvalidId => write!(f, "ID must be greater than 0"),
            ValidationError::EmptyName => write!(f, "Name cannot be empty"),
            ValidationError::NegativeValue => write!(f, "Value cannot be negative"),
            ValidationError::InvalidCategory => write!(f, "Category must be one of: A, B, C"),
        }
    }
}

impl Error for ValidationError {}

pub struct DataProcessor {
    records: HashMap<u32, DataRecord>,
    category_stats: HashMap<String, f64>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: HashMap::new(),
            category_stats: HashMap::new(),
        }
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), ValidationError> {
        if record.id == 0 {
            return Err(ValidationError::InvalidId);
        }
        
        if record.name.trim().is_empty() {
            return Err(ValidationError::EmptyName);
        }
        
        if record.value < 0.0 {
            return Err(ValidationError::NegativeValue);
        }
        
        let valid_categories = ["A", "B", "C"];
        if !valid_categories.contains(&record.category.as_str()) {
            return Err(ValidationError::InvalidCategory);
        }
        
        Ok(())
    }

    pub fn add_record(&mut self, record: DataRecord) -> Result<(), Box<dyn Error>> {
        self.validate_record(&record)?;
        
        if self.records.contains_key(&record.id) {
            return Err("Record with this ID already exists".into());
        }
        
        self.update_category_stats(&record);
        self.records.insert(record.id, record);
        
        Ok(())
    }

    fn update_category_stats(&mut self, record: &DataRecord) {
        let total = self.category_stats
            .entry(record.category.clone())
            .or_insert(0.0);
        *total += record.value;
    }

    pub fn get_category_total(&self, category: &str) -> f64 {
        self.category_stats.get(category).copied().unwrap_or(0.0)
    }

    pub fn get_record(&self, id: u32) -> Option<&DataRecord> {
        self.records.get(&id)
    }

    pub fn transform_values<F>(&mut self, transform_fn: F) 
    where
        F: Fn(f64) -> f64,
    {
        for record in self.records.values_mut() {
            record.value = transform_fn(record.value);
        }
        
        self.recalculate_stats();
    }

    fn recalculate_stats(&mut self) {
        self.category_stats.clear();
        for record in self.records.values() {
            self.update_category_stats(record);
        }
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .values()
            .filter(|record| record.category == category)
            .collect()
    }

    pub fn total_value(&self) -> f64 {
        self.records.values().map(|r| r.value).sum()
    }

    pub fn average_value(&self) -> Option<f64> {
        if self.records.is_empty() {
            None
        } else {
            Some(self.total_value() / self.records.len() as f64)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation() {
        let processor = DataProcessor::new();
        let valid_record = DataRecord {
            id: 1,
            name: "Test".to_string(),
            value: 100.0,
            category: "A".to_string(),
        };
        
        assert!(processor.validate_record(&valid_record).is_ok());
    }

    #[test]
    fn test_invalid_id() {
        let processor = DataProcessor::new();
        let invalid_record = DataRecord {
            id: 0,
            name: "Test".to_string(),
            value: 100.0,
            category: "A".to_string(),
        };
        
        assert!(matches!(
            processor.validate_record(&invalid_record),
            Err(ValidationError::InvalidId)
        ));
    }

    #[test]
    fn test_add_and_retrieve() {
        let mut processor = DataProcessor::new();
        let record = DataRecord {
            id: 1,
            name: "Sample".to_string(),
            value: 50.0,
            category: "B".to_string(),
        };
        
        assert!(processor.add_record(record.clone()).is_ok());
        assert_eq!(processor.get_record(1).unwrap().name, "Sample");
    }

    #[test]
    fn test_category_stats() {
        let mut processor = DataProcessor::new();
        
        let record1 = DataRecord {
            id: 1,
            name: "Item1".to_string(),
            value: 30.0,
            category: "A".to_string(),
        };
        
        let record2 = DataRecord {
            id: 2,
            name: "Item2".to_string(),
            value: 70.0,
            category: "A".to_string(),
        };
        
        processor.add_record(record1).unwrap();
        processor.add_record(record2).unwrap();
        
        assert_eq!(processor.get_category_total("A"), 100.0);
    }

    #[test]
    fn test_value_transformation() {
        let mut processor = DataProcessor::new();
        let record = DataRecord {
            id: 1,
            name: "Test".to_string(),
            value: 10.0,
            category: "C".to_string(),
        };
        
        processor.add_record(record).unwrap();
        processor.transform_values(|x| x * 2.0);
        
        assert_eq!(processor.get_record(1).unwrap().value, 20.0);
    }
}