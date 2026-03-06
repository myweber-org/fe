
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
    InvalidValue,
    InvalidCategory,
    DuplicateRecord,
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataError::InvalidId => write!(f, "Invalid record ID"),
            DataError::InvalidValue => write!(f, "Invalid value field"),
            DataError::InvalidCategory => write!(f, "Invalid category"),
            DataError::DuplicateRecord => write!(f, "Duplicate record found"),
        }
    }
}

impl Error for DataError {}

pub struct DataProcessor {
    records: HashMap<u32, DataRecord>,
    category_stats: HashMap<String, CategoryStats>,
}

#[derive(Debug, Clone)]
pub struct CategoryStats {
    pub count: usize,
    pub total_value: f64,
    pub average_value: f64,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: HashMap::new(),
            category_stats: HashMap::new(),
        }
    }

    pub fn add_record(&mut self, record: DataRecord) -> Result<(), DataError> {
        if record.id == 0 {
            return Err(DataError::InvalidId);
        }

        if record.value < 0.0 || record.value > 10000.0 {
            return Err(DataError::InvalidValue);
        }

        if record.category.is_empty() {
            return Err(DataError::InvalidCategory);
        }

        if self.records.contains_key(&record.id) {
            return Err(DataError::DuplicateRecord);
        }

        self.update_category_stats(&record);
        self.records.insert(record.id, record);
        Ok(())
    }

    fn update_category_stats(&mut self, record: &DataRecord) {
        let stats = self.category_stats
            .entry(record.category.clone())
            .or_insert(CategoryStats {
                count: 0,
                total_value: 0.0,
                average_value: 0.0,
            });

        stats.count += 1;
        stats.total_value += record.value;
        stats.average_value = stats.total_value / stats.count as f64;
    }

    pub fn get_record(&self, id: u32) -> Option<&DataRecord> {
        self.records.get(&id)
    }

    pub fn get_category_stats(&self, category: &str) -> Option<&CategoryStats> {
        self.category_stats.get(category)
    }

    pub fn filter_by_value_range(&self, min: f64, max: f64) -> Vec<&DataRecord> {
        self.records
            .values()
            .filter(|record| record.value >= min && record.value <= max)
            .collect()
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

    pub fn total_records(&self) -> usize {
        self.records.len()
    }

    pub fn average_value(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }
        self.records.values().map(|r| r.value).sum::<f64>() / self.records.len() as f64
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
    fn test_add_valid_record() {
        let mut processor = DataProcessor::new();
        let record = DataRecord {
            id: 1,
            name: "Test".to_string(),
            value: 100.0,
            category: "A".to_string(),
        };

        assert!(processor.add_record(record).is_ok());
        assert_eq!(processor.total_records(), 1);
    }

    #[test]
    fn test_add_invalid_id() {
        let mut processor = DataProcessor::new();
        let record = DataRecord {
            id: 0,
            name: "Test".to_string(),
            value: 100.0,
            category: "A".to_string(),
        };

        assert!(matches!(processor.add_record(record), Err(DataError::InvalidId)));
    }

    #[test]
    fn test_filter_records() {
        let mut processor = DataProcessor::new();
        
        let records = vec![
            DataRecord { id: 1, name: "R1".to_string(), value: 50.0, category: "A".to_string() },
            DataRecord { id: 2, name: "R2".to_string(), value: 150.0, category: "B".to_string() },
            DataRecord { id: 3, name: "R3".to_string(), value: 200.0, category: "A".to_string() },
        ];

        for record in records {
            processor.add_record(record).unwrap();
        }

        let filtered = processor.filter_by_value_range(100.0, 250.0);
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_transform_values() {
        let mut processor = DataProcessor::new();
        let record = DataRecord {
            id: 1,
            name: "Test".to_string(),
            value: 100.0,
            category: "A".to_string(),
        };

        processor.add_record(record).unwrap();
        processor.transform_values(|v| v * 2.0);

        let updated = processor.get_record(1).unwrap();
        assert_eq!(updated.value, 200.0);
    }
}