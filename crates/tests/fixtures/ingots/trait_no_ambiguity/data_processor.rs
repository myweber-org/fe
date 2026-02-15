use csv::Reader;
use serde::Deserialize;
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

pub fn process_data_file(file_path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let mut reader = Reader::from_reader(file);
    let mut records = Vec::new();

    for result in reader.deserialize() {
        let record: Record = result?;
        if record.value >= 0.0 {
            records.push(record);
        }
    }

    Ok(records)
}

pub fn calculate_statistics(records: &[Record]) -> (f64, f64, usize) {
    let sum: f64 = records.iter().map(|r| r.value).sum();
    let count = records.len();
    let average = if count > 0 { sum / count as f64 } else { 0.0 };
    
    (sum, average, count)
}

pub fn filter_by_category(records: Vec<Record>, category: &str) -> Vec<Record> {
    records.into_iter()
        .filter(|r| r.category == category)
        .collect()
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
    EmptyName,
    UnknownCategory,
    DuplicateRecord,
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataError::InvalidId => write!(f, "ID must be greater than zero"),
            DataError::InvalidValue => write!(f, "Value must be between 0.0 and 1000.0"),
            DataError::EmptyName => write!(f, "Name cannot be empty"),
            DataError::UnknownCategory => write!(f, "Category is not recognized"),
            DataError::DuplicateRecord => write!(f, "Record with this ID already exists"),
        }
    }
}

impl Error for DataError {}

pub struct DataProcessor {
    records: HashMap<u32, DataRecord>,
    valid_categories: Vec<String>,
}

impl DataProcessor {
    pub fn new(categories: Vec<String>) -> Self {
        DataProcessor {
            records: HashMap::new(),
            valid_categories: categories,
        }
    }

    pub fn add_record(&mut self, record: DataRecord) -> Result<(), DataError> {
        if record.id == 0 {
            return Err(DataError::InvalidId);
        }

        if self.records.contains_key(&record.id) {
            return Err(DataError::DuplicateRecord);
        }

        if record.name.trim().is_empty() {
            return Err(DataError::EmptyName);
        }

        if record.value < 0.0 || record.value > 1000.0 {
            return Err(DataError::InvalidValue);
        }

        if !self.valid_categories.contains(&record.category) {
            return Err(DataError::UnknownCategory);
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

    pub fn export_csv(&self) -> String {
        let mut output = String::from("id,name,value,category\n");
        
        for record in self.records.values() {
            output.push_str(&format!(
                "{},{},{},{}\n",
                record.id, record.name, record.value, record.category
            ));
        }
        
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_record_addition() {
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
    fn test_invalid_id() {
        let categories = vec!["A".to_string()];
        let mut processor = DataProcessor::new(categories);
        
        let record = DataRecord {
            id: 0,
            name: "Test".to_string(),
            value: 50.0,
            category: "A".to_string(),
        };
        
        assert!(matches!(processor.add_record(record), Err(DataError::InvalidId)));
    }

    #[test]
    fn test_average_calculation() {
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