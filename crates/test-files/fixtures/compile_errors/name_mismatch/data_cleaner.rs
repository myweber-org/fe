
use std::collections::HashMap;

pub struct DataCleaner {
    pub null_placeholder: String,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            null_placeholder: "N/A".to_string(),
        }
    }

    pub fn clean_vector(&self, data: Vec<Option<String>>) -> Vec<String> {
        data.into_iter()
            .map(|item| match item {
                Some(value) if !value.trim().is_empty() => value.trim().to_string(),
                _ => self.null_placeholder.clone(),
            })
            .collect()
    }

    pub fn normalize_strings(strings: &[String]) -> Vec<String> {
        strings
            .iter()
            .map(|s| s.to_lowercase().trim().to_string())
            .collect()
    }

    pub fn count_frequencies(items: &[String]) -> HashMap<String, usize> {
        let mut frequencies = HashMap::new();
        for item in items {
            *frequencies.entry(item.clone()).or_insert(0) += 1;
        }
        frequencies
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_vector() {
        let cleaner = DataCleaner::new();
        let data = vec![
            Some("  hello  ".to_string()),
            None,
            Some("".to_string()),
            Some("world".to_string()),
        ];
        
        let cleaned = cleaner.clean_vector(data);
        assert_eq!(cleaned, vec!["hello", "N/A", "N/A", "world"]);
    }

    #[test]
    fn test_normalize_strings() {
        let strings = vec![
            "  HELLO  ".to_string(),
            "World".to_string(),
            "  TEST  ".to_string(),
        ];
        
        let normalized = DataCleaner::normalize_strings(&strings);
        assert_eq!(normalized, vec!["hello", "world", "test"]);
    }

    #[test]
    fn test_count_frequencies() {
        let items = vec![
            "apple".to_string(),
            "banana".to_string(),
            "apple".to_string(),
            "orange".to_string(),
        ];
        
        let frequencies = DataCleaner::count_frequencies(&items);
        assert_eq!(frequencies.get("apple"), Some(&2));
        assert_eq!(frequencies.get("banana"), Some(&1));
        assert_eq!(frequencies.get("orange"), Some(&1));
    }
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

    pub fn normalize_whitespace(&mut self) {
        for record in self.records.iter_mut() {
            let normalized = record
                .split_whitespace()
                .collect::<Vec<&str>>()
                .join(" ");
            *record = normalized;
        }
    }

    pub fn get_records(&self) -> &Vec<String> {
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
        cleaner.add_record("test".to_string());
        cleaner.add_record("test".to_string());
        cleaner.add_record("another".to_string());

        let unique = cleaner.deduplicate();
        assert_eq!(unique.len(), 2);
        assert_eq!(cleaner.get_records().len(), 2);
    }

    #[test]
    fn test_normalization() {
        let mut cleaner = DataCleaner::new();
        cleaner.add_record("  multiple   spaces   here  ".to_string());
        cleaner.normalize_whitespace();

        assert_eq!(cleaner.get_records()[0], "multiple spaces here");
    }
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
                value: record.value.abs(),
                category: record.category.to_uppercase(),
            };
            wtr.serialize(cleaned_record)?;
        }
    }

    wtr.flush()?;
    Ok(())
}

pub fn validate_record(record: &Record) -> bool {
    !record.name.is_empty() 
        && record.value >= 0.0 
        && !record.category.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_record() {
        let valid_record = Record {
            id: 1,
            name: "Test".to_string(),
            value: 42.5,
            category: "CATEGORY".to_string(),
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

fn clean_data(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let mut reader = Reader::from_path(input_path)?;
    let mut writer = Writer::from_writer(File::create(output_path)?);

    for result in reader.deserialize() {
        let mut record: Record = result?;
        
        record.name = record.name.trim().to_string();
        record.category = record.category.to_lowercase();
        
        if record.value < 0.0 {
            record.value = 0.0;
        }
        
        if record.name.is_empty() {
            continue;
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
    let input = "data/raw.csv";
    let output = "data/cleaned.csv";
    
    clean_data(input, output)?;
    
    let mut reader = Reader::from_path(output)?;
    let mut valid_count = 0;
    
    for result in reader.deserialize() {
        let record: Record = result?;
        if validate_record(&record) {
            valid_count += 1;
        }
    }
    
    println!("Processed {} valid records", valid_count);
    Ok(())
}