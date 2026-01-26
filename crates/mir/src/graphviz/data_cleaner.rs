use csv::{ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};
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
    let reader = BufReader::new(input_file);
    
    let output_file = File::create(Path::new(output_path))?;
    let writer = BufWriter::new(output_file);
    
    let mut csv_reader = ReaderBuilder::new()
        .has_headers(true)
        .from_reader(reader);
    
    let mut csv_writer = WriterBuilder::new()
        .has_headers(true)
        .from_writer(writer);
    
    let mut cleaned_count = 0;
    let mut error_count = 0;
    
    for result in csv_reader.deserialize() {
        match result {
            Ok(mut record: Record) => {
                record.name = record.name.trim().to_string();
                record.category = record.category.to_lowercase();
                
                if record.value < 0.0 {
                    record.value = 0.0;
                }
                
                csv_writer.serialize(&record)?;
                cleaned_count += 1;
            }
            Err(e) => {
                eprintln!("Error processing record: {}", e);
                error_count += 1;
            }
        }
    }
    
    csv_writer.flush()?;
    
    println!("Cleaning completed:");
    println!("  Records processed: {}", cleaned_count);
    println!("  Errors encountered: {}", error_count);
    
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let input_file = "raw_data.csv";
    let output_file = "cleaned_data.csv";
    
    match clean_csv_data(input_file, output_file) {
        Ok(_) => println!("Data cleaning successful"),
        Err(e) => eprintln!("Data cleaning failed: {}", e),
    }
    
    Ok(())
}