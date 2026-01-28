use csv::{ReaderBuilder, WriterBuilder};
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
    let mut rdr = ReaderBuilder::new()
        .has_headers(true)
        .from_reader(input_file);

    let output_file = File::create(Path::new(output_path))?;
    let mut wtr = WriterBuilder::new()
        .has_headers(true)
        .from_writer(output_file);

    for result in rdr.deserialize() {
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
            value: if record.value.is_nan() || record.value.is_infinite() {
                0.0
            } else {
                record.value
            },
            category: if record.category.is_empty() {
                "uncategorized".to_string()
            } else {
                record.category
            },
        };

        wtr.serialize(&cleaned_record)?;
    }

    wtr.flush()?;
    println!("Data cleaning completed successfully");
    Ok(())
}

fn main() {
    let input_file = "raw_data.csv";
    let output_file = "cleaned_data.csv";

    if let Err(e) = clean_csv_data(input_file, output_file) {
        eprintln!("Error occurred during data cleaning: {}", e);
        std::process::exit(1);
    }
}