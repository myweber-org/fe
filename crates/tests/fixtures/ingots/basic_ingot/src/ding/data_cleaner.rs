use csv::{ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    age: u8,
    email: String,
}

impl Record {
    fn is_valid(&self) -> bool {
        !self.name.trim().is_empty()
            && self.age > 0
            && self.email.contains('@')
            && self.email.contains('.')
    }
}

fn clean_csv(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(Path::new(input_path))?;
    let mut rdr = ReaderBuilder::new()
        .has_headers(true)
        .from_reader(input_file);

    let output_file = File::create(Path::new(output_path))?;
    let mut wtr = WriterBuilder::new()
        .has_headers(true)
        .from_writer(output_file);

    let mut valid_count = 0;
    let mut invalid_count = 0;

    for result in rdr.deserialize() {
        let record: Record = result?;
        
        if record.is_valid() {
            wtr.serialize(&record)?;
            valid_count += 1;
        } else {
            invalid_count += 1;
        }
    }

    wtr.flush()?;
    println!("Cleaning complete: {} valid, {} invalid records", valid_count, invalid_count);
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    clean_csv("input.csv", "cleaned_output.csv")
}