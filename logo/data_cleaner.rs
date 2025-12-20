use csv::{ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    age: u32,
    email: String,
}

fn validate_record(record: &Record) -> Result<(), String> {
    if record.name.trim().is_empty() {
        return Err("Name cannot be empty".to_string());
    }
    if record.age > 120 {
        return Err("Age must be less than 120".to_string());
    }
    if !record.email.contains('@') {
        return Err("Invalid email format".to_string());
    }
    Ok(())
}

fn clean_csv(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(Path::new(input_path))?;
    let mut rdr = ReaderBuilder::new().has_headers(true).from_reader(input_file);
    
    let output_file = File::create(Path::new(output_path))?;
    let mut wtr = WriterBuilder::new().has_headers(true).from_writer(output_file);
    
    let mut valid_count = 0;
    let mut invalid_count = 0;
    
    for result in rdr.deserialize() {
        let record: Record = result?;
        
        match validate_record(&record) {
            Ok(_) => {
                wtr.serialize(&record)?;
                valid_count += 1;
            }
            Err(err) => {
                eprintln!("Invalid record ID {}: {}", record.id, err);
                invalid_count += 1;
            }
        }
    }
    
    wtr.flush()?;
    println!("Processing complete: {} valid, {} invalid records", valid_count, invalid_count);
    
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let input = "data/raw_data.csv";
    let output = "data/cleaned_data.csv";
    
    clean_csv(input, output)?;
    
    Ok(())
}