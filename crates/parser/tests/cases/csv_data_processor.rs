
use csv::{ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub category: String,
}

pub fn read_csv_file<P: AsRef<Path>>(file_path: P) -> Result<Vec<DataRecord>, Box<dyn Error>> {
    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .from_path(file_path)?;
    
    let mut records = Vec::new();
    for result in reader.deserialize() {
        let record: DataRecord = result?;
        records.push(record);
    }
    
    Ok(records)
}

pub fn write_csv_file<P: AsRef<Path>>(
    file_path: P,
    records: &[DataRecord],
) -> Result<(), Box<dyn Error>> {
    let mut writer = WriterBuilder::new()
        .has_headers(true)
        .from_path(file_path)?;
    
    for record in records {
        writer.serialize(record)?;
    }
    
    writer.flush()?;
    Ok(())
}

pub fn filter_records_by_category(
    records: &[DataRecord],
    category: &str,
) -> Vec<DataRecord> {
    records
        .iter()
        .filter(|r| r.category == category)
        .cloned()
        .collect()
}

pub fn calculate_average_value(records: &[DataRecord]) -> Option<f64> {
    if records.is_empty() {
        return None;
    }
    
    let sum: f64 = records.iter().map(|r| r.value).sum();
    Some(sum / records.len() as f64)
}

pub fn validate_record(record: &DataRecord) -> Result<(), String> {
    if record.name.trim().is_empty() {
        return Err("Name cannot be empty".to_string());
    }
    
    if record.value < 0.0 {
        return Err("Value cannot be negative".to_string());
    }
    
    if record.category.trim().is_empty() {
        return Err("Category cannot be empty".to_string());
    }
    
    Ok(())
}

pub fn transform_records(records: &[DataRecord]) -> Vec<DataRecord> {
    records
        .iter()
        .map(|r| DataRecord {
            name: r.name.to_uppercase(),
            value: (r.value * 100.0).round() / 100.0,
            ..r.clone()
        })
        .collect()
}