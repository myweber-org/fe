use std::error::Error;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

pub fn merge_csv_files<P: AsRef<Path>>(
    input_paths: &[P],
    output_path: P,
    write_headers: bool,
) -> Result<(), Box<dyn Error>> {
    let mut output_writer = BufWriter::new(File::create(output_path)?);
    let mut first_file = true;

    for input_path in input_paths {
        let mut rdr = csv::Reader::from_path(input_path)?;
        let headers = rdr.headers()?.clone();

        if first_file {
            if write_headers {
                output_writer.write_all(headers.as_bytes())?;
                output_writer.write_all(b"\n")?;
            }
            first_file = false;
        } else if write_headers {
            let current_headers = rdr.headers()?;
            if headers != current_headers {
                return Err("CSV files have different headers".into());
            }
        }

        for result in rdr.records() {
            let record = result?;
            output_writer.write_all(record.as_slice().as_bytes())?;
            output_writer.write_all(b"\n")?;
        }
    }

    output_writer.flush()?;
    Ok(())
}