use csv::{ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    age: u8,
    active: bool,
}

fn clean_data(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let file = File::open(input_path)?;
    let mut rdr = ReaderBuilder::new()
        .has_headers(true)
        .from_reader(file);

    let output_file = File::create(output_path)?;
    let mut wtr = WriterBuilder::new()
        .has_headers(true)
        .from_writer(output_file);

    for result in rdr.deserialize() {
        let record: Record = result?;
        
        if record.age > 0 && record.age < 120 {
            wtr.serialize(Record {
                id: record.id,
                name: record.name.trim().to_string(),
                age: record.age,
                active: record.active,
            })?;
        }
    }

    wtr.flush()?;
    Ok(())
}

fn validate_record(record: &Record) -> bool {
    !record.name.is_empty() && record.age > 0
}

fn main() -> Result<(), Box<dyn Error>> {
    let input = "data/raw.csv";
    let output = "data/cleaned.csv";
    
    clean_data(input, output)?;
    
    let test_file = File::open(output)?;
    let mut test_reader = ReaderBuilder::new()
        .has_headers(true)
        .from_reader(test_file);
        
    for result in test_reader.deserialize() {
        let record: Record = result?;
        if !validate_record(&record) {
            eprintln!("Invalid record found: {:?}", record);
        }
    }
    
    println!("Data cleaning completed successfully");
    Ok(())
}use std::collections::HashSet;

pub struct DataCleaner {
    deduplication_cache: HashSet<String>,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            deduplication_cache: HashSet::new(),
        }
    }

    pub fn normalize_text(&self, input: &str) -> String {
        input.trim().to_lowercase()
    }

    pub fn deduplicate(&mut self, item: &str) -> bool {
        let normalized = self.normalize_text(item);
        if self.deduplication_cache.contains(&normalized) {
            false
        } else {
            self.deduplication_cache.insert(normalized);
            true
        }
    }

    pub fn clean_numeric(&self, input: &str) -> Option<f64> {
        let cleaned: String = input.chars()
            .filter(|c| c.is_numeric() || *c == '.' || *c == '-')
            .collect();
        
        cleaned.parse::<f64>().ok()
    }

    pub fn remove_special_chars(&self, input: &str, keep_chars: &str) -> String {
        input.chars()
            .filter(|c| c.is_alphanumeric() || keep_chars.contains(*c))
            .collect()
    }

    pub fn get_unique_count(&self) -> usize {
        self.deduplication_cache.len()
    }

    pub fn clear_cache(&mut self) {
        self.deduplication_cache.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalization() {
        let cleaner = DataCleaner::new();
        assert_eq!(cleaner.normalize_text("  HELLO World  "), "hello world");
    }

    #[test]
    fn test_deduplication() {
        let mut cleaner = DataCleaner::new();
        assert!(cleaner.deduplicate("Hello"));
        assert!(!cleaner.deduplicate("  HELLO  "));
        assert!(cleaner.deduplicate("World"));
        assert_eq!(cleaner.get_unique_count(), 2);
    }

    #[test]
    fn test_numeric_cleaning() {
        let cleaner = DataCleaner::new();
        assert_eq!(cleaner.clean_numeric("$123.45"), Some(123.45));
        assert_eq!(cleaner.clean_numeric("abc"), None);
    }

    #[test]
    fn test_special_char_removal() {
        let cleaner = DataCleaner::new();
        let result = cleaner.remove_special_chars("hello@world.com", "@.");
        assert_eq!(result, "hello@world.com");
    }
}