use csv::{Reader, Writer};
use std::error::Error;
use std::io;

pub fn filter_numeric_column(input_path: &str, output_path: &str, column_index: usize) -> Result<(), Box<dyn Error>> {
    let mut rdr = Reader::from_path(input_path)?;
    let mut wtr = Writer::from_path(output_path)?;

    let headers = rdr.headers()?.clone();
    wtr.write_record(&headers)?;

    for result in rdr.records() {
        let record = result?;
        if let Some(field) = record.get(column_index) {
            if field.parse::<f64>().is_ok() {
                wtr.write_record(&record)?;
            }
        }
    }

    wtr.flush()?;
    Ok(())
}