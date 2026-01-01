
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct CsvRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub category: String,
}

impl CsvRecord {
    pub fn new(id: u32, name: String, value: f64, category: String) -> Self {
        Self {
            id,
            name,
            value,
            category,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("Name cannot be empty".to_string());
        }
        if self.value < 0.0 {
            return Err("Value must be non-negative".to_string());
        }
        if self.category.is_empty() {
            return Err("Category cannot be empty".to_string());
        }
        Ok(())
    }
}

pub fn process_csv_file<P: AsRef<Path>>(path: P) -> Result<Vec<CsvRecord>, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();
    let mut line_number = 0;

    for line in reader.lines() {
        line_number += 1;
        let line = line?;
        
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }

        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() != 4 {
            return Err(format!("Invalid CSV format at line {}", line_number).into());
        }

        let id = parts[0].parse::<u32>()
            .map_err(|_| format!("Invalid ID at line {}", line_number))?;
        
        let name = parts[1].trim().to_string();
        let value = parts[2].parse::<f64>()
            .map_err(|_| format!("Invalid value at line {}", line_number))?;
        let category = parts[3].trim().to_string();

        let record = CsvRecord::new(id, name, value, category);
        record.validate()
            .map_err(|e| format!("Validation error at line {}: {}", line_number, e))?;
        
        records.push(record);
    }

    Ok(records)
}

pub fn filter_by_category(records: &[CsvRecord], category: &str) -> Vec<CsvRecord> {
    records.iter()
        .filter(|r| r.category == category)
        .cloned()
        .collect()
}

pub fn calculate_total_value(records: &[CsvRecord]) -> f64 {
    records.iter().map(|r| r.value).sum()
}

pub fn find_max_value_record(records: &[CsvRecord]) -> Option<&CsvRecord> {
    records.iter().max_by(|a, b| a.value.partial_cmp(&b.value).unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_processing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "1,Item A,25.5,Electronics").unwrap();
        writeln!(temp_file, "2,Item B,15.0,Books").unwrap();
        writeln!(temp_file, "3,Item C,42.8,Electronics").unwrap();
        
        let records = process_csv_file(temp_file.path()).unwrap();
        assert_eq!(records.len(), 3);
        
        let electronics = filter_by_category(&records, "Electronics");
        assert_eq!(electronics.len(), 2);
        
        let total = calculate_total_value(&records);
        assert!((total - 83.3).abs() < 0.001);
        
        let max_record = find_max_value_record(&records).unwrap();
        assert_eq!(max_record.id, 3);
    }

    #[test]
    fn test_record_validation() {
        let valid_record = CsvRecord::new(1, "Test".to_string(), 10.0, "Category".to_string());
        assert!(valid_record.validate().is_ok());
        
        let invalid_record = CsvRecord::new(2, "".to_string(), -5.0, "".to_string());
        assert!(invalid_record.validate().is_err());
    }
}