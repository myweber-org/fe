use csv::{Reader, Writer};
use std::error::Error;
use std::io;

pub fn filter_numeric_column(input_path: &str, output_path: &str, column_index: usize) -> Result<(), Box<dyn Error>> {
    let mut rdr = Reader::from_path(input_path)?;
    let mut wtr = Writer::from_path(output_path)?;

    let headers = rdr.headers()?.clone();
    wtr.write_record(&headers)?;

    for result in rdr.records() {
        let record = result?;
        if let Some(field) = record.get(column_index) {
            if field.parse::<f64>().is_ok() {
                wtr.write_record(&record)?;
            }
        }
    }

    wtr.flush()?;
    Ok(())
}use std::collections::HashSet;

pub struct DataCleaner {
    records: Vec<String>,
    seen: HashSet<String>,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            records: Vec::new(),
            seen: HashSet::new(),
        }
    }

    pub fn add_record(&mut self, record: String) -> bool {
        if self.is_valid(&record) && !self.seen.contains(&record) {
            self.seen.insert(record.clone());
            self.records.push(record);
            true
        } else {
            false
        }
    }

    pub fn get_unique_records(&self) -> &Vec<String> {
        &self.records
    }

    pub fn clear(&mut self) {
        self.records.clear();
        self.seen.clear();
    }

    fn is_valid(&self, record: &str) -> bool {
        !record.trim().is_empty() && record.len() <= 1000
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplication() {
        let mut cleaner = DataCleaner::new();
        assert!(cleaner.add_record("test1".to_string()));
        assert!(!cleaner.add_record("test1".to_string()));
        assert_eq!(cleaner.get_unique_records().len(), 1);
    }

    #[test]
    fn test_validation() {
        let mut cleaner = DataCleaner::new();
        assert!(!cleaner.add_record("".to_string()));
        assert!(!cleaner.add_record("   ".to_string()));
        assert!(cleaner.add_record("valid".to_string()));
    }
}