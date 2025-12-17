use csv::{ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

fn clean_csv_data(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let reader = BufReader::new(input_file);
    let mut csv_reader = ReaderBuilder::new()
        .has_headers(true)
        .from_reader(reader);

    let output_file = File::create(output_path)?;
    let mut csv_writer = WriterBuilder::new()
        .has_headers(true)
        .from_writer(output_file);

    for result in csv_reader.deserialize() {
        let record: Record = match result {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Skipping invalid record: {}", e);
                continue;
            }
        };

        let cleaned_record = Record {
            id: record.id,
            name: record.name.trim().to_string(),
            value: record.value.abs(),
            category: record.category.to_uppercase(),
        };

        csv_writer.serialize(cleaned_record)?;
    }

    csv_writer.flush()?;
    println!("Data cleaning completed. Output saved to: {}", output_path);
    Ok(())
}

fn validate_file_exists(path: &str) -> bool {
    Path::new(path).exists()
}

fn main() -> Result<(), Box<dyn Error>> {
    let input_file = "raw_data.csv";
    let output_file = "cleaned_data.csv";

    if !validate_file_exists(input_file) {
        eprintln!("Input file '{}' not found", input_file);
        return Ok(());
    }

    match clean_csv_data(input_file, output_file) {
        Ok(_) => println!("Operation successful"),
        Err(e) => eprintln!("Error during processing: {}", e),
    }

    Ok(())
}