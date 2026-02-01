
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
    let mut reader = Reader::from_reader(file);
    
    let mut records = Vec::new();
    for result in reader.deserialize() {
        let record: Record = result?;
        if record.value >= 0.0 {
            records.push(record);
        }
    }
    
    Ok(records)
}

pub fn calculate_statistics(records: &[Record]) -> (f64, f64, usize) {
    let total: f64 = records.iter().map(|r| r.value).sum();
    let average = if records.is_empty() {
        0.0
    } else {
        total / records.len() as f64
    };
    
    let max_value = records
        .iter()
        .map(|r| r.value)
        .fold(f64::NEG_INFINITY, |a, b| a.max(b));
    
    (total, average, max_value as usize)
}

pub fn filter_by_category(records: Vec<Record>, category: &str) -> Vec<Record> {
    records
        .into_iter()
        .filter(|r| r.category == category)
        .collect()
}