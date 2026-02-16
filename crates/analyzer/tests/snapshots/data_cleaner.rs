use csv::{Reader, Writer};
use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use log::{info, warn};

pub struct DataCleaner {
    input_path: String,
    output_path: String,
    delimiter: u8,
}

impl DataCleaner {
    pub fn new(input_path: &str, output_path: &str) -> Self {
        DataCleaner {
            input_path: input_path.to_string(),
            output_path: output_path.to_string(),
            delimiter: b',',
        }
    }

    pub fn set_delimiter(&mut self, delimiter: u8) {
        self.delimiter = delimiter;
    }

    pub fn clean_data(&self) -> Result<(), Box<dyn Error>> {
        info!("Starting data cleaning process");
        
        let input_file = File::open(&self.input_path)?;
        let reader = BufReader::new(input_file);
        let mut csv_reader = Reader::from_reader(reader);
        csv_reader.set_delimiter(self.delimiter);

        let output_file = File::create(&self.output_path)?;
        let mut csv_writer = Writer::from_writer(output_file);

        let headers = csv_reader.headers()?.clone();
        csv_writer.write_record(&headers)?;

        let mut processed_count = 0;
        let mut skipped_count = 0;

        for result in csv_reader.records() {
            match result {
                Ok(record) => {
                    let cleaned_record: Vec<String> = record
                        .iter()
                        .map(|field| field.trim().to_string())
                        .collect();

                    if self.is_valid_record(&cleaned_record) {
                        csv_writer.write_record(&cleaned_record)?;
                        processed_count += 1;
                    } else {
                        warn!("Skipping invalid record: {:?}", cleaned_record);
                        skipped_count += 1;
                    }
                }
                Err(e) => {
                    warn!("Error reading record: {}", e);
                    skipped_count += 1;
                }
            }
        }

        csv_writer.flush()?;
        
        info!("Data cleaning completed. Processed: {}, Skipped: {}", 
              processed_count, skipped_count);
        
        Ok(())
    }

    fn is_valid_record(&self, record: &[String]) -> bool {
        !record.iter().any(|field| field.is_empty())
    }

    pub fn validate_file(&self) -> Result<(), Box<dyn Error>> {
        let file = File::open(&self.input_path)?;
        let reader = BufReader::new(file);
        
        let mut line_count = 0;
        for line in reader.lines() {
            line?;
            line_count += 1;
        }

        if line_count > 0 {
            info!("File validation passed. Total lines: {}", line_count);
            Ok(())
        } else {
            Err("File is empty".into())
        }
    }
}

pub fn initialize_logging() {
    env_logger::init();
    info!("Logging initialized");
}
use std::collections::HashSet;

pub struct DataCleaner {
    pub entries: Vec<String>,
}

impl DataCleaner {
    pub fn new(entries: Vec<String>) -> Self {
        DataCleaner { entries }
    }

    pub fn remove_duplicates(&mut self) {
        let mut seen = HashSet::new();
        self.entries.retain(|entry| seen.insert(entry.clone()));
    }

    pub fn normalize_case(&mut self) {
        for entry in &mut self.entries {
            *entry = entry.to_lowercase();
        }
    }

    pub fn trim_whitespace(&mut self) {
        for entry in &mut self.entries {
            *entry = entry.trim().to_string();
        }
    }

    pub fn clean(&mut self) {
        self.trim_whitespace();
        self.normalize_case();
        self.remove_duplicates();
    }

    pub fn get_results(&self) -> &Vec<String> {
        &self.entries
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cleaner_removes_duplicates() {
        let mut cleaner = DataCleaner::new(vec![
            "apple".to_string(),
            "APPLE".to_string(),
            "banana".to_string(),
            "apple".to_string(),
        ]);
        
        cleaner.clean();
        let results = cleaner.get_results();
        
        assert_eq!(results.len(), 2);
        assert!(results.contains(&"apple".to_string()));
        assert!(results.contains(&"banana".to_string()));
    }

    #[test]
    fn test_cleaner_normalizes_and_trims() {
        let mut cleaner = DataCleaner::new(vec![
            "  Apple  ".to_string(),
            "BANANA".to_string(),
            "  Cherry  ".to_string(),
        ]);
        
        cleaner.clean();
        let results = cleaner.get_results();
        
        assert_eq!(results[0], "apple");
        assert_eq!(results[1], "banana");
        assert_eq!(results[2], "cherry");
    }
}