use csv::{Reader, Writer};
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
    let file = File::open(input_path)?;
    let mut rdr = Reader::from_reader(file);
    let mut wtr = Writer::from_path(output_path)?;

    for result in rdr.deserialize() {
        let mut record: Record = result?;
        
        record.name = record.name.trim().to_string();
        record.category = record.category.to_lowercase();
        
        if record.value < 0.0 {
            record.value = 0.0;
        }
        
        wtr.serialize(&record)?;
    }

    wtr.flush()?;
    Ok(())
}

fn validate_record(record: &Record) -> bool {
    !record.name.is_empty() && record.value.is_finite()
}

fn main() -> Result<(), Box<dyn Error>> {
    let input = "data/raw.csv";
    let output = "data/cleaned.csv";
    
    clean_csv(input, output)?;
    
    let file = File::open(output)?;
    let mut rdr = Reader::from_reader(file);
    
    for result in rdr.deserialize() {
        let record: Record = result?;
        if !validate_record(&record) {
            eprintln!("Invalid record found: {:?}", record);
        }
    }
    
    println!("Data cleaning completed successfully");
    Ok(())
}
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

    pub fn remove_duplicates(&mut self) -> usize {
        let mut seen = HashSet::new();
        let mut unique_records = Vec::new();
        let mut removed_count = 0;

        for record in self.records.drain(..) {
            if seen.insert(record.clone()) {
                unique_records.push(record);
            } else {
                removed_count += 1;
            }
        }

        self.records = unique_records;
        removed_count
    }

    pub fn validate_records(&self) -> (usize, usize) {
        let mut valid_count = 0;
        let mut invalid_count = 0;

        for record in &self.records {
            if !record.trim().is_empty() && record.len() <= 100 {
                valid_count += 1;
            } else {
                invalid_count += 1;
            }
        }

        (valid_count, invalid_count)
    }

    pub fn get_records(&self) -> &[String] {
        &self.records
    }

    pub fn clear(&mut self) {
        self.records.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplication() {
        let mut cleaner = DataCleaner::new();
        cleaner.add_record("test1".to_string());
        cleaner.add_record("test2".to_string());
        cleaner.add_record("test1".to_string());
        
        let removed = cleaner.remove_duplicates();
        assert_eq!(removed, 1);
        assert_eq!(cleaner.get_records().len(), 2);
    }

    #[test]
    fn test_validation() {
        let mut cleaner = DataCleaner::new();
        cleaner.add_record("valid".to_string());
        cleaner.add_record("".to_string());
        
        let (valid, invalid) = cleaner.validate_records();
        assert_eq!(valid, 1);
        assert_eq!(invalid, 1);
    }
}