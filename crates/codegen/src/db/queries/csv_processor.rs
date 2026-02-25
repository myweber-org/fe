use std::error::Error;
use std::fs::File;
use csv::{Reader, Writer};

pub fn filter_csv(input_path: &str, output_path: &str, column_name: &str, filter_value: &str) -> Result<(), Box<dyn Error>> {
    let file = File::open(input_path)?;
    let mut rdr = Reader::from_reader(file);
    let mut wtr = Writer::from_path(output_path)?;

    let headers = rdr.headers()?.clone();
    let column_index = headers.iter()
        .position(|h| h == column_name)
        .ok_or_else(|| format!("Column '{}' not found", column_name))?;

    wtr.write_record(&headers)?;

    for result in rdr.records() {
        let record = result?;
        if record.get(column_index) == Some(filter_value) {
            wtr.write_record(&record)?;
        }
    }

    wtr.flush()?;
    Ok(())
}

pub fn count_csv_rows(file_path: &str) -> Result<usize, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let mut rdr = Reader::from_reader(file);
    let count = rdr.records().count();
    Ok(count)
}