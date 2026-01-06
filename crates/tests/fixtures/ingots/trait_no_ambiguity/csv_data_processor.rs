use csv::{Reader, Writer};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    category: String,
    value: f64,
    active: bool,
}

fn filter_records(records: &[Record], category_filter: &str) -> Vec<Record> {
    records
        .iter()
        .filter(|r| r.category == category_filter && r.active)
        .cloned()
        .collect()
}

fn aggregate_values(records: &[Record]) -> (f64, f64, f64) {
    let sum: f64 = records.iter().map(|r| r.value).sum();
    let count = records.len() as f64;
    let avg = if count > 0.0 { sum / count } else { 0.0 };
    let max = records
        .iter()
        .map(|r| r.value)
        .fold(f64::NEG_INFINITY, f64::max);

    (sum, avg, max)
}

fn process_csv_file(input_path: &str, output_path: &str, category: &str) -> Result<(), Box<dyn Error>> {
    let file = File::open(input_path)?;
    let mut rdr = Reader::from_reader(file);
    
    let mut records = Vec::new();
    for result in rdr.deserialize() {
        let record: Record = result?;
        records.push(record);
    }

    let filtered = filter_records(&records, category);
    let (total, average, maximum) = aggregate_values(&filtered);

    let mut wtr = Writer::from_path(output_path)?;
    for record in &filtered {
        wtr.serialize(record)?;
    }
    wtr.flush()?;

    println!("Processed {} records", filtered.len());
    println!("Total: {:.2}, Average: {:.2}, Maximum: {:.2}", total, average, maximum);

    Ok(())
}

fn generate_sample_data() -> Result<(), Box<dyn Error>> {
    let sample_records = vec![
        Record { id: 1, name: "Item A".to_string(), category: "Electronics".to_string(), value: 299.99, active: true },
        Record { id: 2, name: "Item B".to_string(), category: "Books".to_string(), value: 24.50, active: true },
        Record { id: 3, name: "Item C".to_string(), category: "Electronics".to_string(), value: 599.99, active: false },
        Record { id: 4, name: "Item D".to_string(), category: "Electronics".to_string(), value: 150.00, active: true },
        Record { id: 5, name: "Item E".to_string(), category: "Clothing".to_string(), value: 45.75, active: true },
    ];

    let mut wtr = Writer::from_path("sample_data.csv")?;
    for record in sample_records {
        wtr.serialize(&record)?;
    }
    wtr.flush()?;
    
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    generate_sample_data()?;
    process_csv_file("sample_data.csv", "filtered_electronics.csv", "Electronics")?;
    Ok(())
}