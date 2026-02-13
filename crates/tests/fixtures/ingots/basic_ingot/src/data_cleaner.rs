
use csv::{Reader, Writer};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

fn clean_csv_data(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(Path::new(input_path))?;
    let mut reader = Reader::from_reader(input_file);
    
    let output_file = File::create(Path::new(output_path))?;
    let mut writer = Writer::from_writer(output_file);
    
    let mut cleaned_count = 0;
    let mut skipped_count = 0;
    
    for result in reader.deserialize() {
        let record: Record = match result {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Skipping invalid record: {}", e);
                skipped_count += 1;
                continue;
            }
        };
        
        let cleaned_record = Record {
            id: record.id,
            name: record.name.trim().to_string(),
            value: if record.value.is_finite() { record.value } else { 0.0 },
            category: record.category.to_uppercase(),
        };
        
        writer.serialize(&cleaned_record)?;
        cleaned_count += 1;
    }
    
    writer.flush()?;
    
    println!("Processing complete:");
    println!("  Cleaned records: {}", cleaned_count);
    println!("  Skipped records: {}", skipped_count);
    
    Ok(())
}

fn main() {
    let input_file = "raw_data.csv";
    let output_file = "cleaned_data.csv";
    
    match clean_csv_data(input_file, output_file) {
        Ok(_) => println!("Data cleaning successful"),
        Err(e) => eprintln!("Error during data cleaning: {}", e),
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
    let file = File::open(input_path)?;
    let mut rdr = Reader::from_reader(file);
    let mut wtr = Writer::from_path(output_path)?;

    for result in rdr.deserialize() {
        let mut record: Record = result?;
        
        record.name = record.name.trim().to_string();
        record.category = record.category.to_lowercase();
        
        if record.value < 0.0 {
            record.value = 0.0;
        }
        
        wtr.serialize(&record)?;
    }

    wtr.flush()?;
    Ok(())
}

fn validate_record(record: &Record) -> bool {
    !record.name.is_empty() && record.value >= 0.0
}

fn main() -> Result<(), Box<dyn Error>> {
    let input = "raw_data.csv";
    let output = "cleaned_data.csv";
    
    clean_csv_data(input, output)?;
    
    let file = File::open(output)?;
    let mut rdr = Reader::from_reader(file);
    let mut valid_count = 0;
    let mut total_count = 0;

    for result in rdr.deserialize() {
        let record: Record = result?;
        total_count += 1;
        if validate_record(&record) {
            valid_count += 1;
        }
    }

    println!("Processed {} records", total_count);
    println!("Valid records: {}", valid_count);
    
    Ok(())
}