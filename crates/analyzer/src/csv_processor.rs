
use clap::Parser;
use csv::{ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "csv_filter")]
#[command(about = "Filter CSV rows based on column criteria")]
struct Args {
    #[arg(short, long)]
    input: PathBuf,
    
    #[arg(short, long)]
    output: PathBuf,
    
    #[arg(short, long)]
    column: String,
    
    #[arg(short, long)]
    value: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    name: String,
    age: u32,
    city: String,
    active: bool,
}

fn filter_csv(input_path: &PathBuf, output_path: &PathBuf, column: &str, value: &str) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .from_reader(input_file);

    let output_file = File::create(output_path)?;
    let mut writer = WriterBuilder::new()
        .has_headers(true)
        .from_writer(output_file);

    for result in reader.deserialize() {
        let record: Record = result?;
        
        let matches = match column {
            "name" => record.name == value,
            "age" => record.age.to_string() == value,
            "city" => record.city == value,
            "active" => record.active.to_string() == value,
            _ => false,
        };

        if matches {
            writer.serialize(&record)?;
        }
    }

    writer.flush()?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    
    println!("Processing CSV file: {:?}", args.input);
    println!("Filtering column '{}' for value '{}'", args.column, args.value);
    
    filter_csv(&args.input, &args.output, &args.column, &args.value)?;
    
    println!("Filtered data written to: {:?}", args.output);
    Ok(())
}