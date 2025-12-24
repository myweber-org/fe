
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
    active: bool,
}

fn filter_and_transform_records(
    input_path: &Path,
    output_path: &Path,
    min_value: f64,
) -> Result<(), Box<dyn Error>> {
    let mut reader = Reader::from_path(input_path)?;
    let mut writer = Writer::from_path(output_path)?;

    for result in reader.deserialize() {
        let record: Record = result?;
        
        if record.value >= min_value && record.active {
            let transformed = Record {
                name: record.name.to_uppercase(),
                value: record.value * 1.1,
                ..record
            };
            writer.serialize(transformed)?;
        }
    }

    writer.flush()?;
    Ok(())
}

fn validate_csv_structure(path: &Path) -> Result<(), Box<dyn Error>> {
    let mut reader = Reader::from_path(path)?;
    let headers = reader.headers()?;
    
    let expected = vec!["id", "name", "value", "active"];
    if headers != expected {
        return Err(format!("Invalid CSV structure. Expected {:?}, got {:?}", expected, headers).into());
    }
    
    let mut record_count = 0;
    for result in reader.deserialize::<Record>() {
        let _ = result?;
        record_count += 1;
    }
    
    println!("Validated {} records in {}", record_count, path.display());
    Ok(())
}

fn process_csv_files() -> Result<(), Box<dyn Error>> {
    let input_file = Path::new("data/input.csv");
    let output_file = Path::new("data/output.csv");
    
    validate_csv_structure(input_file)?;
    filter_and_transform_records(input_file, output_file, 50.0)?;
    validate_csv_structure(output_file)?;
    
    println!("Processing completed successfully");
    Ok(())
}

fn main() {
    if let Err(e) = process_csv_files() {
        eprintln!("Error processing CSV files: {}", e);
        std::process::exit(1);
    }
}