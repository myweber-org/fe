use std::error::Error;
use std::fs::File;
use std::path::Path;

pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub category: String,
}

pub fn process_csv_file(file_path: &str) -> Result<Vec<DataRecord>, Box<dyn Error>> {
    let path = Path::new(file_path);
    let file = File::open(path)?;
    let mut rdr = csv::Reader::from_reader(file);
    let mut records = Vec::new();

    for result in rdr.deserialize() {
        let record: DataRecord = result?;
        validate_record(&record)?;
        records.push(record);
    }

    Ok(records)
}

fn validate_record(record: &DataRecord) -> Result<(), String> {
    if record.id == 0 {
        return Err("Invalid ID: zero value not allowed".to_string());
    }
    if record.value < 0.0 {
        return Err("Invalid value: negative number not allowed".to_string());
    }
    if record.category.is_empty() {
        return Err("Invalid category: empty string not allowed".to_string());
    }
    Ok(())
}

pub fn calculate_average(records: &[DataRecord]) -> Option<f64> {
    if records.is_empty() {
        return None;
    }
    let sum: f64 = records.iter().map(|r| r.value).sum();
    Some(sum / records.len() as f64)
}