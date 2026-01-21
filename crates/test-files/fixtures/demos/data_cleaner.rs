use csv::{ReaderBuilder, WriterBuilder};
use std::collections::HashSet;
use std::error::Error;
use std::fs::File;

pub fn remove_duplicates(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let file = File::open(input_path)?;
    let mut rdr = ReaderBuilder::new().has_headers(true).from_reader(file);
    let headers = rdr.headers()?.clone();

    let mut seen = HashSet::new();
    let mut records = Vec::new();

    for result in rdr.records() {
        let record = result?;
        let key: String = record.iter().collect();
        if seen.insert(key) {
            records.push(record);
        }
    }

    let output_file = File::create(output_path)?;
    let mut wtr = WriterBuilder::new().has_headers(true).from_writer(output_file);

    wtr.write_record(&headers)?;
    for record in records {
        wtr.write_record(&record)?;
    }

    wtr.flush()?;
    Ok(())
}
use std::collections::HashSet;

pub struct DataCleaner {
    records: Vec<String>,
    duplicates: HashSet<String>,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            records: Vec::new(),
            duplicates: HashSet::new(),
        }
    }

    pub fn add_record(&mut self, record: String) -> bool {
        if self.duplicates.contains(&record) {
            return false;
        }
        
        if self.records.contains(&record) {
            self.duplicates.insert(record.clone());
            return false;
        }
        
        self.records.push(record);
        true
    }

    pub fn validate_records(&self) -> Vec<&String> {
        self.records
            .iter()
            .filter(|record| !record.trim().is_empty())
            .filter(|record| record.len() <= 255)
            .collect()
    }

    pub fn get_unique_count(&self) -> usize {
        self.records.len()
    }

    pub fn get_duplicate_count(&self) -> usize {
        self.duplicates.len()
    }

    pub fn clear_duplicates(&mut self) {
        self.duplicates.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplication() {
        let mut cleaner = DataCleaner::new();
        assert!(cleaner.add_record("record1".to_string()));
        assert!(!cleaner.add_record("record1".to_string()));
        assert_eq!(cleaner.get_unique_count(), 1);
        assert_eq!(cleaner.get_duplicate_count(), 1);
    }

    #[test]
    fn test_validation() {
        let mut cleaner = DataCleaner::new();
        cleaner.add_record("valid".to_string());
        cleaner.add_record("".to_string());
        
        let valid = cleaner.validate_records();
        assert_eq!(valid.len(), 1);
        assert_eq!(valid[0], "valid");
    }
}