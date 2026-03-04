
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct DataRecord {
    id: u32,
    value: f64,
    category: String,
    valid: bool,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: String) -> Self {
        let valid = value >= 0.0 && !category.is_empty();
        DataRecord {
            id,
            value,
            category,
            valid,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.valid
    }

    pub fn summary(&self) -> String {
        format!("ID: {}, Value: {:.2}, Category: {}", self.id, self.value, self.category)
    }
}

pub fn process_csv_file<P: AsRef<Path>>(path: P) -> Result<Vec<DataRecord>, Box<dyn Error>> {
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
        if parts.len() != 3 {
            eprintln!("Warning: Invalid format at line {}", line_number);
            continue;
        }

        let id = match parts[0].trim().parse::<u32>() {
            Ok(val) => val,
            Err(_) => {
                eprintln!("Warning: Invalid ID at line {}", line_number);
                continue;
            }
        };

        let value = match parts[1].trim().parse::<f64>() {
            Ok(val) => val,
            Err(_) => {
                eprintln!("Warning: Invalid value at line {}", line_number);
                continue;
            }
        };

        let category = parts[2].trim().to_string();
        records.push(DataRecord::new(id, value, category));
    }

    Ok(records)
}

pub fn calculate_statistics(records: &[DataRecord]) -> (f64, f64, usize) {
    let valid_records: Vec<&DataRecord> = records.iter().filter(|r| r.is_valid()).collect();
    
    if valid_records.is_empty() {
        return (0.0, 0.0, 0);
    }

    let sum: f64 = valid_records.iter().map(|r| r.value).sum();
    let count = valid_records.len();
    let mean = sum / count as f64;

    let variance: f64 = valid_records.iter()
        .map(|r| (r.value - mean).powi(2))
        .sum::<f64>() / count as f64;
    let std_dev = variance.sqrt();

    (mean, std_dev, count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_record_creation() {
        let record = DataRecord::new(1, 42.5, "test".to_string());
        assert!(record.is_valid());
        assert_eq!(record.summary(), "ID: 1, Value: 42.50, Category: test");
    }

    #[test]
    fn test_invalid_record() {
        let record = DataRecord::new(2, -5.0, "".to_string());
        assert!(!record.is_valid());
    }

    #[test]
    fn test_csv_processing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "1,42.5,category_a").unwrap();
        writeln!(temp_file, "2,-3.0,category_b").unwrap();
        writeln!(temp_file, "# This is a comment").unwrap();
        writeln!(temp_file, "").unwrap();
        writeln!(temp_file, "3,17.8,category_c").unwrap();

        let records = process_csv_file(temp_file.path()).unwrap();
        assert_eq!(records.len(), 3);
        
        let (mean, std_dev, valid_count) = calculate_statistics(&records);
        assert_eq!(valid_count, 2);
        assert!((mean - 30.15).abs() < 0.01);
        assert!((std_dev - 17.46).abs() < 0.01);
    }
}