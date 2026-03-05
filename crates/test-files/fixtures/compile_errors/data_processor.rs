
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

        if record.value < 0.0 || record.value.is_nan() {
            return Err(DataError::InvalidValue);
        }

        if record.name.is_empty() || record.category.is_empty() {
            return Err(DataError::MissingField);
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

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .values()
            .filter(|record| record.category == category)
            .collect()
    }

    pub fn calculate_average(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }

        let sum: f64 = self.records.values().map(|r| r.value).sum();
        sum / self.records.len() as f64
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

    pub fn export_as_csv(&self) -> String {
        let mut csv = String::from("id,name,value,category\n");
        
        for record in self.records.values() {
            csv.push_str(&format!(
                "{},{},{},{}\n",
                record.id, record.name, record.value, record.category
            ));
        }
        
        csv
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
        assert_eq!(processor.record_count(), 1);
    }

    #[test]
    fn test_duplicate_record() {
        let mut processor = DataProcessor::new();
        let record1 = DataRecord {
            id: 1,
            name: "First".to_string(),
            value: 50.0,
            category: "B".to_string(),
        };
        
        let record2 = DataRecord {
            id: 1,
            name: "Second".to_string(),
            value: 75.0,
            category: "C".to_string(),
        };

        assert!(processor.add_record(record1).is_ok());
        assert!(matches!(
            processor.add_record(record2),
            Err(DataError::DuplicateRecord)
        ));
    }

    #[test]
    fn test_category_totals() {
        let mut processor = DataProcessor::new();
        
        let records = vec![
            DataRecord {
                id: 1,
                name: "Item1".to_string(),
                value: 10.0,
                category: "X".to_string(),
            },
            DataRecord {
                id: 2,
                name: "Item2".to_string(),
                value: 20.0,
                category: "X".to_string(),
            },
            DataRecord {
                id: 3,
                name: "Item3".to_string(),
                value: 15.0,
                category: "Y".to_string(),
            },
        ];

        for record in records {
            processor.add_record(record).unwrap();
        }

        assert_eq!(processor.get_category_total("X"), 30.0);
        assert_eq!(processor.get_category_total("Y"), 15.0);
        assert_eq!(processor.get_category_total("Z"), 0.0);
    }

    #[test]
    fn test_value_transformation() {
        let mut processor = DataProcessor::new();
        let record = DataRecord {
            id: 1,
            name: "Test".to_string(),
            value: 10.0,
            category: "A".to_string(),
        };

        processor.add_record(record).unwrap();
        processor.transform_values(|x| x * 2.0);

        let updated = processor.get_record(1).unwrap();
        assert_eq!(updated.value, 20.0);
        assert_eq!(processor.get_category_total("A"), 20.0);
    }
}