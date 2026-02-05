use clap::Parser;
use std::error::Error;
use std::fs::File;
use std::io::{self, BufWriter, Write};
use csv::{ReaderBuilder, WriterBuilder};

#[derive(Parser, Debug)]
#[command(author, version, about = "Merge multiple CSV files into a single output file.")]
struct Args {
    #[arg(short, long, required = true, help = "Input CSV files to merge")]
    input_files: Vec<String>,
    
    #[arg(short, long, default_value = "merged_output.csv", help = "Output CSV file name")]
    output_file: String,
    
    #[arg(short, long, default_value_t = true, help = "Include headers in output")]
    headers: bool,
}

fn merge_csv_files(inputs: &[String], output: &str, include_headers: bool) -> Result<(), Box<dyn Error>> {
    let mut output_writer = BufWriter::new(File::create(output)?);
    let mut csv_writer = WriterBuilder::new().from_writer(&mut output_writer);
    let mut first_file = true;

    for input_path in inputs {
        let mut csv_reader = ReaderBuilder::new().from_path(input_path)?;
        
        if first_file && include_headers {
            if let Some(headers) = csv_reader.headers().ok() {
                csv_writer.write_record(headers)?;
            }
            first_file = false;
        } else if !first_file && include_headers {
            csv_reader.headers()?;
        }

        for result in csv_reader.records() {
            let record = result?;
            csv_writer.write_record(&record)?;
        }
    }

    csv_writer.flush()?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    
    if args.input_files.is_empty() {
        eprintln!("Error: No input files specified");
        std::process::exit(1);
    }

    println!("Merging {} files into '{}'", args.input_files.len(), args.output_file);
    
    match merge_csv_files(&args.input_files, &args.output_file, args.headers) {
        Ok(_) => {
            println!("Successfully merged CSV files");
            Ok(())
        }
        Err(e) => {
            eprintln!("Error merging files: {}", e);
            std::process::exit(1);
        }
    }
}