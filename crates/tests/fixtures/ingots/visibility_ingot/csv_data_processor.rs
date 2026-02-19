
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

pub fn validate_record(record: &Record) -> Result<(), String> {
    if record.name.trim().is_empty() {
        return Err("Name cannot be empty".to_string());
    }
    
    if record.value < 0.0 {
        return Err("Value must be non-negative".to_string());
    }
    
    if !["A", "B", "C", "D"].contains(&record.category.as_str()) {
        return Err("Category must be A, B, C, or D".to_string());
    }
    
    Ok(())
}

pub fn transform_value(record: &mut Record, multiplier: f64) {
    record.value *= multiplier;
}

pub fn process_csv_file(input_path: &Path, output_path: &Path) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let reader = BufReader::new(input_file);
    
    let mut csv_reader = csv::Reader::from_reader(reader);
    let mut records: Vec<Record> = Vec::new();
    
    for result in csv_reader.deserialize() {
        let mut record: Record = result?;
        
        if let Err(e) = validate_record(&record) {
            eprintln!("Invalid record skipped: {}", e);
            continue;
        }
        
        transform_value(&mut record, 1.5);
        records.push(record);
    }
    
    let output_file = File::create(output_path)?;
    let writer = BufWriter::new(output_file);
    
    let mut csv_writer = csv::Writer::from_writer(writer);
    
    for record in records {
        csv_writer.serialize(record)?;
    }
    
    csv_writer.flush()?;
    
    Ok(())
}

pub fn calculate_statistics(records: &[Record]) -> (f64, f64, f64) {
    if records.is_empty() {
        return (0.0, 0.0, 0.0);
    }
    
    let sum: f64 = records.iter().map(|r| r.value).sum();
    let count = records.len() as f64;
    let mean = sum / count;
    
    let variance: f64 = records.iter()
        .map(|r| (r.value - mean).powi(2))
        .sum::<f64>() / count;
    
    let std_dev = variance.sqrt();
    
    (sum, mean, std_dev)
}