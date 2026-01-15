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
}