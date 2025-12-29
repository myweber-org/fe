
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
}