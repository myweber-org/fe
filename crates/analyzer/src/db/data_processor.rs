
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub metadata: HashMap<String, String>,
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

pub fn validate_record(record: &DataRecord) -> Result<(), ValidationError> {
    if record.id == 0 {
        return Err(ValidationError::InvalidId);
    }
    
    if record.name.trim().is_empty() {
        return Err(ValidationError::EmptyName);
    }
    
    if record.value < 0.0 {
        return Err(ValidationError::NegativeValue);
    }
    
    if !record.metadata.contains_key("source") {
        return Err(ValidationError::MissingMetadata("source".to_string()));
    }
    
    Ok(())
}

pub fn transform_record(record: &DataRecord) -> DataRecord {
    let mut transformed = record.clone();
    transformed.name = record.name.to_uppercase();
    transformed.value = (record.value * 100.0).round() / 100.0;
    
    let mut new_metadata = record.metadata.clone();
    new_metadata.insert("processed".to_string(), "true".to_string());
    new_metadata.insert("original_value".to_string(), record.value.to_string());
    
    transformed.metadata = new_metadata;
    transformed
}

pub fn process_records(records: Vec<DataRecord>) -> Result<Vec<DataRecord>, ValidationError> {
    let mut processed_records = Vec::with_capacity(records.len());
    
    for record in records {
        validate_record(&record)?;
        let processed = transform_record(&record);
        processed_records.push(processed);
    }
    
    Ok(processed_records)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_validate_valid_record() {
        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), "test".to_string());
        
        let record = DataRecord {
            id: 1,
            name: "Test Record".to_string(),
            value: 42.5,
            metadata,
        };
        
        assert!(validate_record(&record).is_ok());
    }
    
    #[test]
    fn test_validate_invalid_id() {
        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), "test".to_string());
        
        let record = DataRecord {
            id: 0,
            name: "Test Record".to_string(),
            value: 42.5,
            metadata,
        };
        
        assert!(matches!(validate_record(&record), Err(ValidationError::InvalidId)));
    }
    
    #[test]
    fn test_transform_record() {
        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), "test".to_string());
        
        let record = DataRecord {
            id: 1,
            name: "test record".to_string(),
            value: 42.56789,
            metadata,
        };
        
        let transformed = transform_record(&record);
        
        assert_eq!(transformed.name, "TEST RECORD");
        assert_eq!(transformed.value, 42.57);
        assert_eq!(transformed.metadata.get("processed"), Some(&"true".to_string()));
        assert_eq!(transformed.metadata.get("original_value"), Some(&"42.56789".to_string()));
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;

use serde::{Deserialize, Serialize};
use csv::{ReaderBuilder, WriterBuilder};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Record {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub category: String,
}

impl Record {
    pub fn new(id: u32, name: String, value: f64, category: String) -> Self {
        Self {
            id,
            name,
            value,
            category,
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.name.is_empty() && self.value >= 0.0
    }
}

pub struct DataProcessor {
    records: Vec<Record>,
}

impl DataProcessor {
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
        }
    }

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<usize, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut csv_reader = ReaderBuilder::new()
            .has_headers(true)
            .from_reader(reader);

        let mut count = 0;
        for result in csv_reader.deserialize() {
            let record: Record = result?;
            if record.is_valid() {
                self.records.push(record);
                count += 1;
            }
        }

        Ok(count)
    }

    pub fn save_to_csv<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn Error>> {
        let file = File::create(path)?;
        let writer = BufWriter::new(file);
        let mut csv_writer = WriterBuilder::new()
            .has_headers(true)
            .from_writer(writer);

        for record in &self.records {
            csv_writer.serialize(record)?;
        }

        csv_writer.flush()?;
        Ok(())
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<Record> {
        self.records
            .iter()
            .filter(|r| r.category == category)
            .cloned()
            .collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }

        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn add_record(&mut self, record: Record) {
        if record.is_valid() {
            self.records.push(record);
        }
    }

    pub fn get_records(&self) -> &[Record] {
        &self.records
    }

    pub fn clear(&mut self) {
        self.records.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_record_validation() {
        let valid_record = Record::new(1, "Test".to_string(), 10.5, "A".to_string());
        assert!(valid_record.is_valid());

        let invalid_record = Record::new(2, "".to_string(), -5.0, "B".to_string());
        assert!(!invalid_record.is_valid());
    }

    #[test]
    fn test_data_processor_operations() {
        let mut processor = DataProcessor::new();
        
        processor.add_record(Record::new(1, "Item1".to_string(), 10.0, "CategoryA".to_string()));
        processor.add_record(Record::new(2, "Item2".to_string(), 20.0, "CategoryB".to_string()));
        processor.add_record(Record::new(3, "Item3".to_string(), 30.0, "CategoryA".to_string()));

        assert_eq!(processor.get_records().len(), 3);
        
        let category_a = processor.filter_by_category("CategoryA");
        assert_eq!(category_a.len(), 2);

        let avg = processor.calculate_average();
        assert_eq!(avg, Some(20.0));
    }

    #[test]
    fn test_csv_operations() -> Result<(), Box<dyn Error>> {
        let mut processor = DataProcessor::new();
        processor.add_record(Record::new(1, "Test1".to_string(), 15.5, "Cat1".to_string()));
        processor.add_record(Record::new(2, "Test2".to_string(), 25.5, "Cat2".to_string()));

        let temp_file = NamedTempFile::new()?;
        let path = temp_file.path();

        processor.save_to_csv(path)?;
        
        let mut new_processor = DataProcessor::new();
        let count = new_processor.load_from_csv(path)?;
        
        assert_eq!(count, 2);
        assert_eq!(new_processor.get_records().len(), 2);
        
        Ok(())
    }
}