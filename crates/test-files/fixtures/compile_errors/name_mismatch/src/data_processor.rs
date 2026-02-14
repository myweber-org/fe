
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
    MissingField,
    DuplicateRecord,
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataError::InvalidId => write!(f, "Invalid record ID"),
            DataError::InvalidValue => write!(f, "Invalid numeric value"),
            DataError::MissingField => write!(f, "Required field is missing"),
            DataError::DuplicateRecord => write!(f, "Duplicate record detected"),
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
    pub total_records: usize,
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

        if record.value < 0.0 {
            return Err(DataError::InvalidValue);
        }

        if record.name.is_empty() || record.category.is_empty() {
            return Err(DataError::MissingField);
        }

        if self.records.contains_key(&record.id) {
            return Err(DataError::DuplicateRecord);
        }

        self.records.insert(record.id, record.clone());
        self.update_category_stats(&record);

        Ok(())
    }

    pub fn get_record(&self, id: u32) -> Option<&DataRecord> {
        self.records.get(&id)
    }

    pub fn get_category_stats(&self, category: &str) -> Option<&CategoryStats> {
        self.category_stats.get(category)
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .values()
            .filter(|record| record.category == category)
            .collect()
    }

    pub fn calculate_total_value(&self) -> f64 {
        self.records.values().map(|record| record.value).sum()
    }

    pub fn get_top_categories(&self, limit: usize) -> Vec<(String, CategoryStats)> {
        let mut categories: Vec<_> = self.category_stats.clone().into_iter().collect();
        categories.sort_by(|a, b| {
            b.1.total_value
                .partial_cmp(&a.1.total_value)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        categories.into_iter().take(limit).collect()
    }

    fn update_category_stats(&mut self, record: &DataRecord) {
        let stats = self
            .category_stats
            .entry(record.category.clone())
            .or_insert(CategoryStats {
                total_records: 0,
                total_value: 0.0,
                average_value: 0.0,
            });

        stats.total_records += 1;
        stats.total_value += record.value;
        stats.average_value = stats.total_value / stats.total_records as f64;
    }

    pub fn transform_records<F>(&mut self, transform_fn: F)
    where
        F: Fn(&mut DataRecord),
    {
        for record in self.records.values_mut() {
            transform_fn(record);
        }
        self.recalculate_stats();
    }

    fn recalculate_stats(&mut self) {
        self.category_stats.clear();
        for record in self.records.values() {
            self.update_category_stats(record);
        }
    }

    pub fn validate_all_records(&self) -> Vec<(u32, DataError)> {
        let mut errors = Vec::new();

        for (id, record) in &self.records {
            if record.id == 0 {
                errors.push((*id, DataError::InvalidId));
            }
            if record.value < 0.0 {
                errors.push((*id, DataError::InvalidValue));
            }
            if record.name.is_empty() || record.category.is_empty() {
                errors.push((*id, DataError::MissingField));
            }
        }

        errors
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
        assert_eq!(processor.records.len(), 1);
    }

    #[test]
    fn test_add_invalid_record() {
        let mut processor = DataProcessor::new();
        let record = DataRecord {
            id: 0,
            name: "Test".to_string(),
            value: 100.0,
            category: "A".to_string(),
        };

        assert!(processor.add_record(record).is_err());
    }

    #[test]
    fn test_category_stats() {
        let mut processor = DataProcessor::new();

        let records = vec![
            DataRecord {
                id: 1,
                name: "Record1".to_string(),
                value: 50.0,
                category: "CategoryA".to_string(),
            },
            DataRecord {
                id: 2,
                name: "Record2".to_string(),
                value: 100.0,
                category: "CategoryA".to_string(),
            },
            DataRecord {
                id: 3,
                name: "Record3".to_string(),
                value: 75.0,
                category: "CategoryB".to_string(),
            },
        ];

        for record in records {
            processor.add_record(record).unwrap();
        }

        let stats = processor.get_category_stats("CategoryA").unwrap();
        assert_eq!(stats.total_records, 2);
        assert_eq!(stats.total_value, 150.0);
        assert_eq!(stats.average_value, 75.0);
    }

    #[test]
    fn test_filter_records() {
        let mut processor = DataProcessor::new();

        let records = vec![
            DataRecord {
                id: 1,
                name: "A1".to_string(),
                value: 10.0,
                category: "TypeA".to_string(),
            },
            DataRecord {
                id: 2,
                name: "B1".to_string(),
                value: 20.0,
                category: "TypeB".to_string(),
            },
            DataRecord {
                id: 3,
                name: "A2".to_string(),
                value: 30.0,
                category: "TypeA".to_string(),
            },
        ];

        for record in records {
            processor.add_record(record).unwrap();
        }

        let filtered = processor.filter_by_category("TypeA");
        assert_eq!(filtered.len(), 2);
    }
}