
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Record {
    pub id: u32,
    pub category: String,
    pub value: f64,
    pub active: bool,
}

pub fn load_csv(file_path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut csv_reader = csv::Reader::from_reader(reader);
    
    let mut records = Vec::new();
    for result in csv_reader.deserialize() {
        let record: Record = result?;
        records.push(record);
    }
    
    Ok(records)
}

pub fn filter_active_records(records: &[Record]) -> Vec<Record> {
    records.iter()
        .filter(|r| r.active)
        .cloned()
        .collect()
}

pub fn aggregate_by_category(records: &[Record]) -> HashMap<String, f64> {
    let mut aggregates = HashMap::new();
    
    for record in records {
        let entry = aggregates.entry(record.category.clone()).or_insert(0.0);
        *entry += record.value;
    }
    
    aggregates
}

pub fn save_results(results: &HashMap<String, f64>, output_path: &str) -> Result<(), Box<dyn Error>> {
    let file = File::create(output_path)?;
    let writer = BufWriter::new(file);
    let mut csv_writer = csv::Writer::from_writer(writer);
    
    for (category, total) in results {
        csv_writer.serialize((category, total))?;
    }
    
    csv_writer.flush()?;
    Ok(())
}

pub fn process_csv_data(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let records = load_csv(input_path)?;
    let active_records = filter_active_records(&records);
    let aggregated_data = aggregate_by_category(&active_records);
    save_results(&aggregated_data, output_path)?;
    
    println!("Processed {} records", records.len());
    println!("Active records: {}", active_records.len());
    println!("Categories aggregated: {}", aggregated_data.len());
    
    Ok(())
}