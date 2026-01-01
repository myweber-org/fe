use csv::Reader;
use serde::Deserialize;
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

pub fn process_data_file(file_path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let mut rdr = Reader::from_reader(file);
    let mut records = Vec::new();

    for result in rdr.deserialize() {
        let record: Record = result?;
        if record.value < 0.0 {
            return Err(format!("Invalid value {} for record {}", record.value, record.id).into());
        }
        records.push(record);
    }

    Ok(records)
}

pub fn calculate_statistics(records: &[Record]) -> (f64, f64, usize) {
    let total: f64 = records.iter().map(|r| r.value).sum();
    let average = total / records.len() as f64;
    let max_value = records.iter().map(|r| r.value).fold(0.0, f64::max);
    let category_count = records.iter().map(|r| &r.category).collect::<std::collections::HashSet<_>>().len();

    (average, max_value, category_count)
}