
use csv::{ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::io::{self, BufReader, BufWriter};
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

fn clean_csv_data(input_path: &Path, output_path: &Path) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let reader = BufReader::new(input_file);
    let mut csv_reader = ReaderBuilder::new()
        .has_headers(true)
        .from_reader(reader);

    let output_file = File::create(output_path)?;
    let writer = BufWriter::new(output_file);
    let mut csv_writer = WriterBuilder::new()
        .has_headers(true)
        .from_writer(writer);

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

        csv_writer.serialize(&cleaned_record)?;
    }

    csv_writer.flush()?;
    println!("Data cleaning completed successfully");
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let input_path = Path::new("input_data.csv");
    let output_path = Path::new("cleaned_data.csv");

    if !input_path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "Input file not found",
        ))?;
    }

    clean_csv_data(input_path, output_path)
}