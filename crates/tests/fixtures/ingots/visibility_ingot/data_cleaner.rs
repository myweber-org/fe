use std::collections::HashSet;
use std::iter::FromIterator;

pub struct DataCleaner {
    records: Vec<String>,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            records: Vec::new(),
        }
    }

    pub fn add_record(&mut self, record: &str) {
        self.records.push(record.to_string());
    }

    pub fn deduplicate(&mut self) -> Vec<String> {
        let unique_set: HashSet<String> = HashSet::from_iter(self.records.drain(..));
        let mut unique_vec: Vec<String> = unique_set.into_iter().collect();
        unique_vec.sort();
        self.records = unique_vec.clone();
        unique_vec
    }

    pub fn normalize_whitespace(&self) -> Vec<String> {
        self.records
            .iter()
            .map(|s| s.split_whitespace().collect::<Vec<&str>>().join(" "))
            .collect()
    }

    pub fn to_lowercase(&self) -> Vec<String> {
        self.records.iter().map(|s| s.to_lowercase()).collect()
    }

    pub fn count(&self) -> usize {
        self.records.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplication() {
        let mut cleaner = DataCleaner::new();
        cleaner.add_record("apple");
        cleaner.add_record("banana");
        cleaner.add_record("apple");
        cleaner.add_record("cherry");
        
        let unique = cleaner.deduplicate();
        assert_eq!(unique, vec!["apple", "banana", "cherry"]);
        assert_eq!(cleaner.count(), 3);
    }

    #[test]
    fn test_normalization() {
        let mut cleaner = DataCleaner::new();
        cleaner.add_record("  multiple   spaces   here  ");
        cleaner.add_record("mixed\tTABS\nand newlines");
        
        let normalized = cleaner.normalize_whitespace();
        assert_eq!(normalized[0], "multiple spaces here");
        assert_eq!(normalized[1], "mixed TABS and newlines");
    }
}use csv::{ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

fn clean_data(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .from_reader(input_file);

    let output_file = File::create(output_path)?;
    let mut writer = WriterBuilder::new()
        .has_headers(true)
        .from_writer(output_file);

    for result in reader.deserialize() {
        let mut record: Record = result?;
        
        record.name = record.name.trim().to_string();
        record.category = record.category.to_lowercase();
        
        if record.value < 0.0 {
            record.value = 0.0;
        }
        
        writer.serialize(&record)?;
    }

    writer.flush()?;
    Ok(())
}

fn validate_record(record: &Record) -> bool {
    !record.name.is_empty() &&
    record.value >= 0.0 &&
    !record.category.is_empty()
}

fn main() -> Result<(), Box<dyn Error>> {
    let input = "data/raw.csv";
    let output = "data/cleaned.csv";
    
    match clean_data(input, output) {
        Ok(_) => println!("Data cleaning completed successfully"),
        Err(e) => eprintln!("Error cleaning data: {}", e),
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_clean_data() {
        let input_data = "id,name,value,category\n1,  John  ,-5.0,TECH\n2,Jane,10.5,SCIENCE\n";
        
        let mut input_file = NamedTempFile::new().unwrap();
        write!(input_file, "{}", input_data).unwrap();
        
        let output_file = NamedTempFile::new().unwrap();
        
        let result = clean_data(input_file.path().to_str().unwrap(), output_file.path().to_str().unwrap());
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_record() {
        let valid_record = Record {
            id: 1,
            name: "Test".to_string(),
            value: 10.0,
            category: "tech".to_string(),
        };
        
        let invalid_record = Record {
            id: 2,
            name: "".to_string(),
            value: -5.0,
            category: "".to_string(),
        };
        
        assert!(validate_record(&valid_record));
        assert!(!validate_record(&invalid_record));
    }
}