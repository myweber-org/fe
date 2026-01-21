
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
    category_stats: HashMap<String, CategoryStats>,
}

#[derive(Debug, Clone)]
pub struct CategoryStats {
    pub total_value: f64,
    pub record_count: usize,
    pub average_value: f64,
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
        
        self.records.insert(record.id, record.clone());
        self.update_category_stats(&record);
        
        Ok(())
    }

    pub fn get_record(&self, id: u32) -> Option<&DataRecord> {
        self.records.get(&id)
    }

    pub fn remove_record(&mut self, id: u32) -> Option<DataRecord> {
        if let Some(record) = self.records.remove(&id) {
            self.recalculate_stats();
            Some(record)
        } else {
            None
        }
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

    pub fn get_category_stats(&self, category: &str) -> Option<&CategoryStats> {
        self.category_stats.get(category)
    }

    pub fn get_total_value(&self) -> f64 {
        self.records.values().map(|r| r.value).sum()
    }

    pub fn get_average_value(&self) -> Option<f64> {
        let count = self.records.len();
        if count > 0 {
            Some(self.get_total_value() / count as f64)
        } else {
            None
        }
    }

    fn update_category_stats(&mut self, record: &DataRecord) {
        let stats = self.category_stats
            .entry(record.category.clone())
            .or_insert(CategoryStats {
                total_value: 0.0,
                record_count: 0,
                average_value: 0.0,
            });
        
        stats.total_value += record.value;
        stats.record_count += 1;
        stats.average_value = stats.total_value / stats.record_count as f64;
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
            .filter(|r| r.category == category)
            .collect()
    }

    pub fn get_all_records(&self) -> Vec<&DataRecord> {
        self.records.values().collect()
    }
}

impl Default for DataProcessor {
    fn default() -> Self {
        Self::new()
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
        
        let invalid_record = DataRecord {
            id: 0,
            name: "".to_string(),
            value: -10.0,
            category: "D".to_string(),
        };
        
        assert!(processor.validate_record(&invalid_record).is_err());
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
        assert_eq!(processor.get_record(2), None);
    }

    #[test]
    fn test_category_stats() {
        let mut processor = DataProcessor::new();
        
        let records = vec![
            DataRecord { id: 1, name: "R1".to_string(), value: 10.0, category: "A".to_string() },
            DataRecord { id: 2, name: "R2".to_string(), value: 20.0, category: "A".to_string() },
            DataRecord { id: 3, name: "R3".to_string(), value: 30.0, category: "B".to_string() },
        ];
        
        for record in records {
            processor.add_record(record).unwrap();
        }
        
        let stats_a = processor.get_category_stats("A").unwrap();
        assert_eq!(stats_a.total_value, 30.0);
        assert_eq!(stats_a.record_count, 2);
        assert_eq!(stats_a.average_value, 15.0);
        
        let stats_b = processor.get_category_stats("B").unwrap();
        assert_eq!(stats_b.total_value, 30.0);
        assert_eq!(stats_b.record_count, 1);
        assert_eq!(stats_b.average_value, 30.0);
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
        
        let updated = processor.get_record(1).unwrap();
        assert_eq!(updated.value, 20.0);
    }
}