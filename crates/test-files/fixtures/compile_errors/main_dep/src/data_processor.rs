use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub category: String,
}

pub fn process_csv_file(file_path: &str) -> Result<Vec<DataRecord>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();

    for (line_number, line) in reader.lines().enumerate() {
        let line = line?;
        
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }

        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() != 3 {
            return Err(format!("Invalid format at line {}", line_number + 1).into());
        }

        let id = parts[0].parse::<u32>()
            .map_err(|_| format!("Invalid ID at line {}", line_number + 1))?;
        
        let value = parts[1].parse::<f64>()
            .map_err(|_| format!("Invalid value at line {}", line_number + 1))?;
        
        let category = parts[2].trim().to_string();
        
        if category.is_empty() {
            return Err(format!("Empty category at line {}", line_number + 1).into());
        }

        records.push(DataRecord { id, value, category });
    }

    if records.is_empty() {
        return Err("No valid records found in file".into());
    }

    Ok(records)
}

pub fn calculate_statistics(records: &[DataRecord]) -> (f64, f64, f64) {
    let count = records.len() as f64;
    let sum: f64 = records.iter().map(|r| r.value).sum();
    let mean = sum / count;
    
    let variance: f64 = records.iter()
        .map(|r| (r.value - mean).powi(2))
        .sum::<f64>() / count;
    
    let std_dev = variance.sqrt();
    
    (mean, variance, std_dev)
}

pub fn filter_by_category(records: Vec<DataRecord>, category: &str) -> Vec<DataRecord> {
    records.into_iter()
        .filter(|r| r.category == category)
        .collect()
}