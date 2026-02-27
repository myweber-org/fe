use std::collections::HashSet;

pub struct DataCleaner {
    records: Vec<String>,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            records: Vec::new(),
        }
    }

    pub fn add_record(&mut self, record: String) {
        self.records.push(record);
    }

    pub fn deduplicate(&mut self) -> usize {
        let unique_set: HashSet<String> = self.records.drain(..).collect();
        let original_count = self.records.len();
        self.records = unique_set.into_iter().collect();
        original_count - self.records.len()
    }

    pub fn validate_records(&self) -> Vec<bool> {
        self.records
            .iter()
            .map(|record| !record.trim().is_empty() && record.len() <= 255)
            .collect()
    }

    pub fn get_clean_records(&self) -> Vec<&str> {
        self.records.iter().map(|s| s.as_str()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplication() {
        let mut cleaner = DataCleaner::new();
        cleaner.add_record("test".to_string());
        cleaner.add_record("test".to_string());
        cleaner.add_record("unique".to_string());
        
        let removed = cleaner.deduplicate();
        assert_eq!(removed, 1);
        assert_eq!(cleaner.records.len(), 2);
    }

    #[test]
    fn test_validation() {
        let mut cleaner = DataCleaner::new();
        cleaner.add_record("valid".to_string());
        cleaner.add_record("".to_string());
        cleaner.add_record("a".repeat(256));
        
        let results = cleaner.validate_records();
        assert_eq!(results, vec![true, false, false]);
    }
}use csv::{Reader, Writer};
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

fn clean_csv(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let mut reader = Reader::from_path(input_path)?;
    let mut writer = Writer::from_writer(File::create(output_path)?);

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
    let input_file = "raw_data.csv";
    let output_file = "cleaned_data.csv";
    
    clean_csv(input_file, output_file)?;
    
    let mut reader = Reader::from_path(output_file)?;
    let mut valid_count = 0;
    let mut total_count = 0;
    
    for result in reader.deserialize() {
        let record: Record = result?;
        total_count += 1;
        
        if validate_record(&record) {
            valid_count += 1;
        }
    }
    
    println!("Processed {} records", total_count);
    println!("Valid records: {}", valid_count);
    
    Ok(())
}
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

pub fn clean_csv_data(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let file = File::open(input_path)?;
    let mut rdr = Reader::from_reader(file);
    let mut wtr = csv::Writer::from_path(output_path)?;

    for result in rdr.deserialize() {
        let record: Record = result?;
        
        if record.value.is_finite() && !record.name.is_empty() {
            let cleaned_record = Record {
                id: record.id,
                name: record.name.trim().to_string(),
                value: record.value.max(0.0),
                category: record.category.to_uppercase(),
            };
            wtr.serialize(cleaned_record)?;
        }
    }

    wtr.flush()?;
    Ok(())
}

pub fn validate_record(record: &Record) -> bool {
    record.id > 0 && 
    !record.name.is_empty() && 
    record.value >= 0.0 && 
    !record.category.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_validate_record() {
        let valid_record = Record {
            id: 1,
            name: "Test".to_string(),
            value: 10.5,
            category: "CATEGORY".to_string(),
        };
        assert!(validate_record(&valid_record));
    }

    #[test]
    fn test_clean_csv_data() {
        let input_data = "id,name,value,category\n1,test,10.5,cat";
        let input_file = NamedTempFile::new().unwrap();
        std::fs::write(input_file.path(), input_data).unwrap();
        
        let output_file = NamedTempFile::new().unwrap();
        
        let result = clean_csv_data(
            input_file.path().to_str().unwrap(),
            output_file.path().to_str().unwrap()
        );
        
        assert!(result.is_ok());
    }
}