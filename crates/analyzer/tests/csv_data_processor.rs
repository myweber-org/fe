
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, PartialEq)]
pub struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

impl Record {
    pub fn new(id: u32, name: String, value: f64, category: String) -> Result<Self, String> {
        if name.is_empty() {
            return Err("Name cannot be empty".to_string());
        }
        if value < 0.0 {
            return Err("Value must be non-negative".to_string());
        }
        if category.is_empty() {
            return Err("Category cannot be empty".to_string());
        }
        
        Ok(Self {
            id,
            name,
            value,
            category,
        })
    }
    
    pub fn transform_value(&mut self, multiplier: f64) {
        self.value *= multiplier;
    }
    
    pub fn get_category(&self) -> &str {
        &self.category
    }
    
    pub fn get_value(&self) -> f64 {
        self.value
    }
}

pub fn process_csv_file(file_path: &Path) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();
    let mut line_number = 0;
    
    for line in reader.lines() {
        line_number += 1;
        let line_content = line?;
        
        if line_content.trim().is_empty() || line_content.starts_with('#') {
            continue;
        }
        
        let parts: Vec<&str> = line_content.split(',').collect();
        if parts.len() != 4 {
            return Err(format!("Invalid CSV format at line {}", line_number).into());
        }
        
        let id = parts[0].parse::<u32>()
            .map_err(|e| format!("Invalid ID at line {}: {}", line_number, e))?;
        
        let name = parts[1].trim().to_string();
        
        let value = parts[2].parse::<f64>()
            .map_err(|e| format!("Invalid value at line {}: {}", line_number, e))?;
        
        let category = parts[3].trim().to_string();
        
        match Record::new(id, name, value, category) {
            Ok(record) => records.push(record),
            Err(e) => return Err(format!("Validation error at line {}: {}", line_number, e).into()),
        }
    }
    
    Ok(records)
}

pub fn calculate_total_by_category(records: &[Record]) -> std::collections::HashMap<String, f64> {
    let mut totals = std::collections::HashMap::new();
    
    for record in records {
        let entry = totals.entry(record.get_category().to_string()).or_insert(0.0);
        *entry += record.get_value();
    }
    
    totals
}

pub fn filter_records_by_threshold(records: &[Record], threshold: f64) -> Vec<&Record> {
    records.iter()
        .filter(|r| r.get_value() >= threshold)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_record_creation() {
        let record = Record::new(1, "Test".to_string(), 100.0, "A".to_string());
        assert!(record.is_ok());
        
        let invalid_record = Record::new(2, "".to_string(), -10.0, "".to_string());
        assert!(invalid_record.is_err());
    }
    
    #[test]
    fn test_value_transformation() {
        let mut record = Record::new(1, "Test".to_string(), 100.0, "A".to_string()).unwrap();
        record.transform_value(1.5);
        assert_eq!(record.get_value(), 150.0);
    }
    
    #[test]
    fn test_csv_processing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "1,Item1,100.0,CategoryA").unwrap();
        writeln!(temp_file, "2,Item2,200.0,CategoryB").unwrap();
        writeln!(temp_file, "# This is a comment").unwrap();
        writeln!(temp_file, "").unwrap();
        writeln!(temp_file, "3,Item3,300.0,CategoryA").unwrap();
        
        let records = process_csv_file(temp_file.path()).unwrap();
        assert_eq!(records.len(), 3);
        
        let totals = calculate_total_by_category(&records);
        assert_eq!(totals.get("CategoryA"), Some(&400.0));
        assert_eq!(totals.get("CategoryB"), Some(&200.0));
        
        let filtered = filter_records_by_threshold(&records, 150.0);
        assert_eq!(filtered.len(), 2);
    }
}