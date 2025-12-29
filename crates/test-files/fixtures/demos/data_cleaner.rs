use csv::{ReaderBuilder, WriterBuilder};
use std::collections::HashSet;
use std::error::Error;
use std::fs::File;

pub fn remove_duplicates(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let file = File::open(input_path)?;
    let mut rdr = ReaderBuilder::new().has_headers(true).from_reader(file);
    let headers = rdr.headers()?.clone();

    let mut seen = HashSet::new();
    let mut records = Vec::new();

    for result in rdr.records() {
        let record = result?;
        let key: String = record.iter().collect();
        if seen.insert(key) {
            records.push(record);
        }
    }

    let output_file = File::create(output_path)?;
    let mut wtr = WriterBuilder::new().has_headers(true).from_writer(output_file);

    wtr.write_record(&headers)?;
    for record in records {
        wtr.write_record(&record)?;
    }

    wtr.flush()?;
    Ok(())
}