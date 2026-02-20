
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

#[derive(Debug)]
pub struct ProcessedRecord {
    pub id: u32,
    pub normalized_name: String,
    pub adjusted_value: f64,
    pub category_code: u8,
}

pub fn read_csv_records<P: AsRef<Path>>(path: P) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut csv_reader = csv::Reader::from_reader(reader);
    
    let mut records = Vec::new();
    for result in csv_reader.deserialize() {
        let record: Record = result?;
        records.push(record);
    }
    
    Ok(records)
}

pub fn validate_record(record: &Record) -> Result<(), String> {
    if record.name.trim().is_empty() {
        return Err("Name cannot be empty".to_string());
    }
    
    if record.value < 0.0 {
        return Err("Value cannot be negative".to_string());
    }
    
    if !record.category.chars().all(|c| c.is_alphabetic()) {
        return Err("Category must contain only letters".to_string());
    }
    
    Ok(())
}

pub fn process_record(record: Record) -> Result<ProcessedRecord, String> {
    validate_record(&record)?;
    
    let normalized_name = record.name.to_uppercase();
    let adjusted_value = record.value * 1.1;
    let category_code = match record.category.as_str() {
        "A" | "ALPHA" => 1,
        "B" | "BETA" => 2,
        "G" | "GAMMA" => 3,
        _ => 0,
    };
    
    Ok(ProcessedRecord {
        id: record.id,
        normalized_name,
        adjusted_value,
        category_code,
    })
}

pub fn write_processed_records<P: AsRef<Path>>(
    records: &[ProcessedRecord],
    path: P,
) -> Result<(), Box<dyn Error>> {
    let file = File::create(path)?;
    let writer = BufWriter::new(file);
    let mut csv_writer = csv::Writer::from_writer(writer);
    
    for record in records {
        csv_writer.serialize(record)?;
    }
    
    csv_writer.flush()?;
    Ok(())
}

pub fn process_csv_file(input_path: &str, output_path: &str) -> Result<usize, Box<dyn Error>> {
    let records = read_csv_records(input_path)?;
    let mut processed_records = Vec::new();
    let mut error_count = 0;
    
    for record in records {
        match process_record(record) {
            Ok(processed) => processed_records.push(processed),
            Err(e) => {
                eprintln!("Failed to process record: {}", e);
                error_count += 1;
            }
        }
    }
    
    write_processed_records(&processed_records, output_path)?;
    
    Ok(processed_records.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_validate_record_valid() {
        let record = Record {
            id: 1,
            name: "Test".to_string(),
            value: 100.0,
            category: "Alpha".to_string(),
        };
        
        assert!(validate_record(&record).is_ok());
    }
    
    #[test]
    fn test_validate_record_invalid_name() {
        let record = Record {
            id: 1,
            name: "   ".to_string(),
            value: 100.0,
            category: "Alpha".to_string(),
        };
        
        assert!(validate_record(&record).is_err());
    }
    
    #[test]
    fn test_process_record() {
        let record = Record {
            id: 42,
            name: "example".to_string(),
            value: 50.0,
            category: "BETA".to_string(),
        };
        
        let processed = process_record(record).unwrap();
        assert_eq!(processed.id, 42);
        assert_eq!(processed.normalized_name, "EXAMPLE");
        assert_eq!(processed.adjusted_value, 55.0);
        assert_eq!(processed.category_code, 2);
    }
    
    #[test]
    fn test_csv_roundtrip() {
        let records = vec![
            Record {
                id: 1,
                name: "First".to_string(),
                value: 10.0,
                category: "A".to_string(),
            },
            Record {
                id: 2,
                name: "Second".to_string(),
                value: 20.0,
                category: "B".to_string(),
            },
        ];
        
        let input_file = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();
        
        let mut writer = csv::Writer::from_writer(&input_file);
        for record in &records {
            writer.serialize(record).unwrap();
        }
        writer.flush().unwrap();
        
        let processed_count = process_csv_file(
            input_file.path().to_str().unwrap(),
            output_file.path().to_str().unwrap(),
        ).unwrap();
        
        assert_eq!(processed_count, 2);
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug)]
pub struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

pub fn load_csv(file_path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();

    for (index, line) in reader.lines().enumerate() {
        let line = line?;
        if index == 0 {
            continue;
        }

        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() >= 4 {
            let record = Record {
                id: parts[0].parse()?,
                name: parts[1].to_string(),
                value: parts[2].parse()?,
                category: parts[3].to_string(),
            };
            records.push(record);
        }
    }

    Ok(records)
}

pub fn filter_by_category(records: &[Record], category: &str) -> Vec<&Record> {
    records
        .iter()
        .filter(|record| record.category == category)
        .collect()
}

pub fn calculate_average(records: &[&Record]) -> Option<f64> {
    if records.is_empty() {
        return None;
    }

    let sum: f64 = records.iter().map(|r| r.value).sum();
    Some(sum / records.len() as f64)
}

pub fn find_max_value(records: &[Record]) -> Option<&Record> {
    records.iter().max_by(|a, b| a.value.partial_cmp(&b.value).unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_by_category() {
        let records = vec![
            Record {
                id: 1,
                name: "Item A".to_string(),
                value: 10.5,
                category: "Electronics".to_string(),
            },
            Record {
                id: 2,
                name: "Item B".to_string(),
                value: 25.0,
                category: "Books".to_string(),
            },
        ];

        let filtered = filter_by_category(&records, "Electronics");
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "Item A");
    }

    #[test]
    fn test_calculate_average() {
        let records = vec![
            Record {
                id: 1,
                name: "Test".to_string(),
                value: 10.0,
                category: "Test".to_string(),
            },
            Record {
                id: 2,
                name: "Test".to_string(),
                value: 20.0,
                category: "Test".to_string(),
            },
        ];

        let refs: Vec<&Record> = records.iter().collect();
        let avg = calculate_average(&refs).unwrap();
        assert_eq!(avg, 15.0);
    }
}use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug, Clone)]
pub struct Record {
    pub id: u32,
    pub category: String,
    pub value: f64,
    pub active: bool,
}

pub struct DataProcessor {
    records: Vec<Record>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_from_csv(&mut self, path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        
        for (index, line) in reader.lines().enumerate() {
            if index == 0 {
                continue;
            }
            
            let line = line?;
            let parts: Vec<&str> = line.split(',').collect();
            
            if parts.len() >= 4 {
                let record = Record {
                    id: parts[0].parse()?,
                    category: parts[1].to_string(),
                    value: parts[2].parse()?,
                    active: parts[3].parse().unwrap_or(false),
                };
                self.records.push(record);
            }
        }
        
        Ok(())
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<Record> {
        self.records
            .iter()
            .filter(|r| r.category == category)
            .cloned()
            .collect()
    }

    pub fn filter_active(&self) -> Vec<Record> {
        self.records
            .iter()
            .filter(|r| r.active)
            .cloned()
            .collect()
    }

    pub fn aggregate_by_category(&self) -> HashMap<String, f64> {
        let mut aggregates = HashMap::new();
        
        for record in &self.records {
            let entry = aggregates.entry(record.category.clone()).or_insert(0.0);
            *entry += record.value;
        }
        
        aggregates
    }

    pub fn calculate_average(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }
        
        let total: f64 = self.records.iter().map(|r| r.value).sum();
        total / self.records.len() as f64
    }

    pub fn find_max_value(&self) -> Option<&Record> {
        self.records.iter().max_by(|a, b| {
            a.value.partial_cmp(&b.value).unwrap()
        })
    }

    pub fn get_statistics(&self) -> (usize, f64, f64) {
        let count = self.records.len();
        let avg = self.calculate_average();
        let max = self.find_max_value().map(|r| r.value).unwrap_or(0.0);
        
        (count, avg, max)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,category,value,active").unwrap();
        writeln!(temp_file, "1,electronics,250.50,true").unwrap();
        writeln!(temp_file, "2,clothing,89.99,true").unwrap();
        writeln!(temp_file, "3,electronics,150.00,false").unwrap();
        
        let mut processor = DataProcessor::new();
        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        
        assert!(result.is_ok());
        assert_eq!(processor.records.len(), 3);
        
        let electronics = processor.filter_by_category("electronics");
        assert_eq!(electronics.len(), 2);
        
        let active_items = processor.filter_active();
        assert_eq!(active_items.len(), 2);
        
        let aggregates = processor.aggregate_by_category();
        assert_eq!(aggregates.get("electronics"), Some(&400.5));
        
        let stats = processor.get_statistics();
        assert_eq!(stats.0, 3);
    }
}