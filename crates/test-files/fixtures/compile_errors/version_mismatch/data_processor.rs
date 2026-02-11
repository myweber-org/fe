
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
pub enum DataError {
    InvalidId,
    InvalidName,
    InvalidValue,
    InvalidCategory,
    DuplicateRecord,
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataError::InvalidId => write!(f, "Invalid record ID"),
            DataError::InvalidName => write!(f, "Invalid record name"),
            DataError::InvalidValue => write!(f, "Invalid record value"),
            DataError::InvalidCategory => write!(f, "Invalid record category"),
            DataError::DuplicateRecord => write!(f, "Duplicate record detected"),
        }
    }
}

impl Error for DataError {}

pub struct DataProcessor {
    records: HashMap<u32, DataRecord>,
    category_totals: HashMap<String, f64>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: HashMap::new(),
            category_totals: HashMap::new(),
        }
    }

    pub fn add_record(&mut self, record: DataRecord) -> Result<(), DataError> {
        if record.id == 0 {
            return Err(DataError::InvalidId);
        }
        
        if record.name.trim().is_empty() {
            return Err(DataError::InvalidName);
        }
        
        if record.value < 0.0 || record.value > 10000.0 {
            return Err(DataError::InvalidValue);
        }
        
        if record.category.trim().is_empty() {
            return Err(DataError::InvalidCategory);
        }
        
        if self.records.contains_key(&record.id) {
            return Err(DataError::DuplicateRecord);
        }
        
        self.records.insert(record.id, record.clone());
        
        let total = self.category_totals
            .entry(record.category.clone())
            .or_insert(0.0);
        *total += record.value;
        
        Ok(())
    }

    pub fn get_record(&self, id: u32) -> Option<&DataRecord> {
        self.records.get(&id)
    }

    pub fn get_category_total(&self, category: &str) -> f64 {
        *self.category_totals.get(category).unwrap_or(&0.0)
    }

    pub fn calculate_average_value(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }
        
        let sum: f64 = self.records.values().map(|r| r.value).sum();
        sum / self.records.len() as f64
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .values()
            .filter(|record| record.category == category)
            .collect()
    }

    pub fn transform_values<F>(&mut self, transform_fn: F) 
    where
        F: Fn(f64) -> f64,
    {
        for record in self.records.values_mut() {
            record.value = transform_fn(record.value);
        }
        
        self.recalculate_totals();
    }

    fn recalculate_totals(&mut self) {
        self.category_totals.clear();
        
        for record in self.records.values() {
            let total = self.category_totals
                .entry(record.category.clone())
                .or_insert(0.0);
            *total += record.value;
        }
    }

    pub fn record_count(&self) -> usize {
        self.records.len()
    }

    pub fn clear(&mut self) {
        self.records.clear();
        self.category_totals.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_valid_record() {
        let mut processor = DataProcessor::new();
        let record = DataRecord {
            id: 1,
            name: String::from("Test Record"),
            value: 100.0,
            category: String::from("A"),
        };
        
        assert!(processor.add_record(record).is_ok());
        assert_eq!(processor.record_count(), 1);
    }

    #[test]
    fn test_add_duplicate_record() {
        let mut processor = DataProcessor::new();
        let record1 = DataRecord {
            id: 1,
            name: String::from("Record 1"),
            value: 100.0,
            category: String::from("A"),
        };
        
        let record2 = DataRecord {
            id: 1,
            name: String::from("Record 2"),
            value: 200.0,
            category: String::from("B"),
        };
        
        assert!(processor.add_record(record1).is_ok());
        assert!(processor.add_record(record2).is_err());
    }

    #[test]
    fn test_category_totals() {
        let mut processor = DataProcessor::new();
        
        let records = vec![
            DataRecord {
                id: 1,
                name: String::from("Item 1"),
                value: 50.0,
                category: String::from("Electronics"),
            },
            DataRecord {
                id: 2,
                name: String::from("Item 2"),
                value: 75.0,
                category: String::from("Electronics"),
            },
            DataRecord {
                id: 3,
                name: String::from("Item 3"),
                value: 25.0,
                category: String::from("Books"),
            },
        ];
        
        for record in records {
            processor.add_record(record).unwrap();
        }
        
        assert_eq!(processor.get_category_total("Electronics"), 125.0);
        assert_eq!(processor.get_category_total("Books"), 25.0);
        assert_eq!(processor.get_category_total("Clothing"), 0.0);
    }

    #[test]
    fn test_value_transformation() {
        let mut processor = DataProcessor::new();
        
        let record = DataRecord {
            id: 1,
            name: String::from("Test Item"),
            value: 100.0,
            category: String::from("Test"),
        };
        
        processor.add_record(record).unwrap();
        
        processor.transform_values(|x| x * 1.1);
        
        let updated_record = processor.get_record(1).unwrap();
        assert_eq!(updated_record.value, 110.0);
        assert_eq!(processor.get_category_total("Test"), 110.0);
    }
}