use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub category: String,
    pub timestamp: String,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: String, timestamp: String) -> Self {
        DataRecord {
            id,
            value,
            category,
            timestamp,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.id > 0 && self.value >= 0.0 && !self.category.is_empty()
    }
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<usize, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut count = 0;

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            
            if line_num == 0 {
                continue;
            }

            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() != 4 {
                continue;
            }

            let id = match parts[0].parse::<u32>() {
                Ok(val) => val,
                Err(_) => continue,
            };

            let value = match parts[1].parse::<f64>() {
                Ok(val) => val,
                Err(_) => continue,
            };

            let category = parts[2].to_string();
            let timestamp = parts[3].to_string();

            let record = DataRecord::new(id, value, category, timestamp);
            if record.is_valid() {
                self.records.push(record);
                count += 1;
            }
        }

        Ok(count)
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }

        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn get_statistics(&self) -> (f64, f64, f64) {
        if self.records.is_empty() {
            return (0.0, 0.0, 0.0);
        }

        let values: Vec<f64> = self.records.iter().map(|r| r.value).collect();
        let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let avg = self.calculate_average().unwrap_or(0.0);

        (min, max, avg)
    }

    pub fn count_records(&self) -> usize {
        self.records.len()
    }

    pub fn clear(&mut self) {
        self.records.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_validation() {
        let valid_record = DataRecord::new(1, 10.5, "A".to_string(), "2024-01-01".to_string());
        assert!(valid_record.is_valid());

        let invalid_record = DataRecord::new(0, -5.0, "".to_string(), "2024-01-01".to_string());
        assert!(!invalid_record.is_valid());
    }

    #[test]
    fn test_processor_operations() {
        let mut processor = DataProcessor::new();
        assert_eq!(processor.count_records(), 0);

        let record1 = DataRecord::new(1, 10.0, "A".to_string(), "2024-01-01".to_string());
        let record2 = DataRecord::new(2, 20.0, "B".to_string(), "2024-01-02".to_string());
        
        processor.records.push(record1);
        processor.records.push(record2);
        
        assert_eq!(processor.count_records(), 2);
        assert_eq!(processor.filter_by_category("A").len(), 1);
        assert_eq!(processor.calculate_average(), Some(15.0));
        
        let (min, max, avg) = processor.get_statistics();
        assert_eq!(min, 10.0);
        assert_eq!(max, 20.0);
        assert_eq!(avg, 15.0);
        
        processor.clear();
        assert_eq!(processor.count_records(), 0);
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
pub enum DataError {
    InvalidId,
    InvalidValue,
    MissingField,
    DuplicateRecord,
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataError::InvalidId => write!(f, "ID must be greater than 0"),
            DataError::InvalidValue => write!(f, "Value must be positive"),
            DataError::MissingField => write!(f, "Required field is missing"),
            DataError::DuplicateRecord => write!(f, "Record with this ID already exists"),
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

        if record.value <= 0.0 {
            return Err(DataError::InvalidValue);
        }

        if record.name.is_empty() || record.category.is_empty() {
            return Err(DataError::MissingField);
        }

        if self.records.contains_key(&record.id) {
            return Err(DataError::DuplicateRecord);
        }

        let category_total = self.category_totals
            .entry(record.category.clone())
            .or_insert(0.0);
        *category_total += record.value;

        self.records.insert(record.id, record);
        Ok(())
    }

    pub fn get_record(&self, id: u32) -> Option<&DataRecord> {
        self.records.get(&id)
    }

    pub fn get_category_total(&self, category: &str) -> f64 {
        *self.category_totals.get(category).unwrap_or(&0.0)
    }

    pub fn apply_discount(&mut self, category: &str, discount_percent: f64) -> Result<(), DataError> {
        if discount_percent < 0.0 || discount_percent > 100.0 {
            return Err(DataError::InvalidValue);
        }

        let multiplier = 1.0 - (discount_percent / 100.0);
        
        for record in self.records.values_mut() {
            if record.category == category {
                record.value *= multiplier;
            }
        }

        if let Some(total) = self.category_totals.get_mut(category) {
            *total *= multiplier;
        }

        Ok(())
    }

    pub fn total_value(&self) -> f64 {
        self.records.values().map(|r| r.value).sum()
    }

    pub fn record_count(&self) -> usize {
        self.records.len()
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
            name: String::from("Test Item"),
            value: 100.0,
            category: String::from("Electronics"),
        };

        assert!(processor.add_record(record).is_ok());
        assert_eq!(processor.record_count(), 1);
    }

    #[test]
    fn test_add_invalid_record() {
        let mut processor = DataProcessor::new();
        let record = DataRecord {
            id: 0,
            name: String::from("Invalid Item"),
            value: 50.0,
            category: String::from("Test"),
        };

        assert!(processor.add_record(record).is_err());
    }

    #[test]
    fn test_category_totals() {
        let mut processor = DataProcessor::new();
        
        let record1 = DataRecord {
            id: 1,
            name: String::from("Item 1"),
            value: 100.0,
            category: String::from("Electronics"),
        };

        let record2 = DataRecord {
            id: 2,
            name: String::from("Item 2"),
            value: 200.0,
            category: String::from("Electronics"),
        };

        processor.add_record(record1).unwrap();
        processor.add_record(record2).unwrap();

        assert_eq!(processor.get_category_total("Electronics"), 300.0);
        assert_eq!(processor.total_value(), 300.0);
    }

    #[test]
    fn test_apply_discount() {
        let mut processor = DataProcessor::new();
        
        let record = DataRecord {
            id: 1,
            name: String::from("Test Item"),
            value: 100.0,
            category: String::from("Electronics"),
        };

        processor.add_record(record).unwrap();
        processor.apply_discount("Electronics", 10.0).unwrap();

        assert_eq!(processor.get_category_total("Electronics"), 90.0);
    }
}