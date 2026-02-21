use std::collections::HashSet;

pub struct DataCleaner {
    deduplication_enabled: bool,
    normalization_enabled: bool,
}

impl DataCleaner {
    pub fn new(deduplication: bool, normalization: bool) -> Self {
        DataCleaner {
            deduplication_enabled: deduplication,
            normalization_enabled: normalization,
        }
    }

    pub fn clean_dataset(&self, data: Vec<String>) -> Vec<String> {
        let mut processed_data = data;

        if self.deduplication_enabled {
            processed_data = Self::remove_duplicates(processed_data);
        }

        if self.normalization_enabled {
            processed_data = Self::normalize_entries(processed_data);
        }

        processed_data
    }

    fn remove_duplicates(data: Vec<String>) -> Vec<String> {
        let mut seen = HashSet::new();
        data.into_iter()
            .filter(|item| seen.insert(item.clone()))
            .collect()
    }

    fn normalize_entries(data: Vec<String>) -> Vec<String> {
        data.into_iter()
            .map(|entry| {
                entry.trim()
                    .to_lowercase()
                    .chars()
                    .filter(|c| c.is_alphanumeric() || c.is_whitespace())
                    .collect()
            })
            .collect()
    }

    pub fn statistics(&self, original: &[String], cleaned: &[String]) -> (usize, usize, f64) {
        let original_len = original.len();
        let cleaned_len = cleaned.len();
        let reduction = if original_len > 0 {
            (original_len - cleaned_len) as f64 / original_len as f64 * 100.0
        } else {
            0.0
        };
        (original_len, cleaned_len, reduction)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplication() {
        let cleaner = DataCleaner::new(true, false);
        let data = vec![
            "apple".to_string(),
            "banana".to_string(),
            "apple".to_string(),
            "cherry".to_string(),
        ];
        let cleaned = cleaner.clean_dataset(data);
        assert_eq!(cleaned.len(), 3);
        assert!(cleaned.contains(&"apple".to_string()));
    }

    #[test]
    fn test_normalization() {
        let cleaner = DataCleaner::new(false, true);
        let data = vec!["  APPLE  ".to_string(), "Banana!".to_string()];
        let cleaned = cleaner.clean_dataset(data);
        assert_eq!(cleaned[0], "apple");
        assert_eq!(cleaned[1], "banana");
    }

    #[test]
    fn test_statistics() {
        let cleaner = DataCleaner::new(true, false);
        let original = vec![
            "apple".to_string(),
            "apple".to_string(),
            "banana".to_string(),
        ];
        let cleaned = cleaner.clean_dataset(original.clone());
        let stats = cleaner.statistics(&original, &cleaned);
        assert_eq!(stats.0, 3);
        assert_eq!(stats.1, 2);
        assert!((stats.2 - 33.33).abs() < 0.1);
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

fn clean_csv_data(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let mut reader = Reader::from_reader(input_file);
    
    let output_file = File::create(output_path)?;
    let mut writer = Writer::from_writer(output_file);

    for result in reader.deserialize() {
        let mut record: Record = result?;
        
        record.name = record.name.trim().to_string();
        record.category = record.category.to_lowercase();
        
        if record.value < 0.0 {
            record.value = 0.0;
        }
        
        if record.name.is_empty() {
            record.name = "Unknown".to_string();
        }
        
        writer.serialize(&record)?;
    }
    
    writer.flush()?;
    Ok(())
}

fn validate_record(record: &Record) -> bool {
    !record.name.is_empty() && record.value >= 0.0
}

fn main() -> Result<(), Box<dyn Error>> {
    let input_file = "raw_data.csv";
    let output_file = "cleaned_data.csv";
    
    match clean_csv_data(input_file, output_file) {
        Ok(_) => println!("Data cleaning completed successfully"),
        Err(e) => eprintln!("Error during data cleaning: {}", e),
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_clean_csv_data() {
        let input_data = "id,name,value,category\n1,  John  ,-5.0,TECH\n2,,10.5,SCIENCE\n";
        
        let mut input_file = NamedTempFile::new().unwrap();
        write!(input_file, "{}", input_data).unwrap();
        
        let output_file = NamedTempFile::new().unwrap();
        
        let result = clean_csv_data(
            input_file.path().to_str().unwrap(),
            output_file.path().to_str().unwrap()
        );
        
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_validate_record() {
        let valid_record = Record {
            id: 1,
            name: "Test".to_string(),
            value: 10.0,
            category: "test".to_string(),
        };
        
        let invalid_record = Record {
            id: 2,
            name: "".to_string(),
            value: -5.0,
            category: "test".to_string(),
        };
        
        assert!(validate_record(&valid_record));
        assert!(!validate_record(&invalid_record));
    }
}