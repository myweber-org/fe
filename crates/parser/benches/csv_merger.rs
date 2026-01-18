use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;

pub fn merge_csv_files<P: AsRef<Path>>(
    input_paths: &[P],
    output_path: P,
    write_headers: bool,
) -> Result<(), Box<dyn Error>> {
    let mut output_writer = csv::Writer::from_writer(BufWriter::new(File::create(output_path)?));
    let mut headers_written = false;

    for (index, input_path) in input_paths.iter().enumerate() {
        let file = File::open(input_path)?;
        let mut reader = csv::Reader::from_reader(BufReader::new(file));

        if index == 0 && write_headers {
            if let Some(headers) = reader.headers().ok() {
                output_writer.write_record(headers)?;
                headers_written = true;
            }
        }

        for result in reader.records() {
            let record = result?;
            output_writer.write_record(&record)?;
        }
    }

    if !headers_written && write_headers {
        eprintln!("Warning: Headers were requested but no headers found in input files.");
    }

    output_writer.flush()?;
    Ok(())
}