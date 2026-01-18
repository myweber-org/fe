use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use csv::{ReaderBuilder, WriterBuilder};

pub fn clean_csv(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let reader = BufReader::new(input_file);
    let mut csv_reader = ReaderBuilder::new().has_headers(true).from_reader(reader);
    
    let output_file = File::create(output_path)?;
    let writer = BufWriter::new(output_file);
    let mut csv_writer = WriterBuilder::new().has_headers(true).from_writer(writer);
    
    let headers = csv_reader.headers()?.clone();
    csv_writer.write_record(&headers)?;
    
    for result in csv_reader.records() {
        let record = result?;
        let cleaned_record: Vec<String> = record
            .iter()
            .map(|field| {
                if field.trim().is_empty() || field.eq_ignore_ascii_case("null") {
                    String::from("")
                } else {
                    field.to_string()
                }
            })
            .collect();
        
        csv_writer.write_record(&cleaned_record)?;
    }
    
    csv_writer.flush()?;
    Ok(())
}use std::collections::HashSet;

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

    pub fn deduplicate(&mut self) -> Vec<String> {
        let mut seen = HashSet::new();
        let mut unique_records = Vec::new();

        for record in self.records.drain(..) {
            if seen.insert(record.clone()) {
                unique_records.push(record);
            }
        }

        self.records = unique_records.clone();
        unique_records
    }

    pub fn validate_records(&self) -> Result<usize, &'static str> {
        if self.records.is_empty() {
            return Err("No records to validate");
        }

        let mut valid_count = 0;
        for record in &self.records {
            if !record.trim().is_empty() && record.len() <= 1000 {
                valid_count += 1;
            }
        }

        Ok(valid_count)
    }

    pub fn get_statistics(&self) -> (usize, usize) {
        let total = self.records.len();
        let avg_length = if total > 0 {
            self.records.iter().map(|r| r.len()).sum::<usize>() / total
        } else {
            0
        };
        (total, avg_length)
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

        let deduped = cleaner.deduplicate();
        assert_eq!(deduped.len(), 2);
        assert!(deduped.contains(&"test".to_string()));
        assert!(deduped.contains(&"unique".to_string()));
    }

    #[test]
    fn test_validation() {
        let mut cleaner = DataCleaner::new();
        cleaner.add_record("valid".to_string());
        cleaner.add_record("".to_string());

        let valid_count = cleaner.validate_records().unwrap();
        assert_eq!(valid_count, 1);
    }
}