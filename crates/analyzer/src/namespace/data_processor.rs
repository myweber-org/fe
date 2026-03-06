
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
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u64,
    pub timestamp: i64,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

impl DataRecord {
    pub fn new(id: u64, timestamp: i64) -> Self {
        Self {
            id,
            timestamp,
            values: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn add_value(&mut self, value: f64) -> &mut Self {
        self.values.push(value);
        self
    }

    pub fn add_metadata(&mut self, key: &str, value: &str) -> &mut Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.id == 0 {
            return Err("Invalid record ID".to_string());
        }
        
        if self.timestamp < 0 {
            return Err("Timestamp cannot be negative".to_string());
        }
        
        if self.values.is_empty() {
            return Err("Record must contain at least one value".to_string());
        }
        
        for value in &self.values {
            if !value.is_finite() {
                return Err("Values must be finite numbers".to_string());
            }
        }
        
        Ok(())
    }

    pub fn calculate_statistics(&self) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        
        if self.values.is_empty() {
            return stats;
        }
        
        let sum: f64 = self.values.iter().sum();
        let count = self.values.len() as f64;
        let mean = sum / count;
        
        let variance: f64 = self.values.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / count;
        
        stats.insert("mean".to_string(), mean);
        stats.insert("sum".to_string(), sum);
        stats.insert("count".to_string(), count);
        stats.insert("variance".to_string(), variance);
        
        if let Some(&min) = self.values.iter().min_by(|a, b| a.partial_cmp(b).unwrap()) {
            stats.insert("min".to_string(), min);
        }
        
        if let Some(&max) = self.values.iter().max_by(|a, b| a.partial_cmp(b).unwrap()) {
            stats.insert("max".to_string(), max);
        }
        
        stats
    }
}

pub fn process_records(records: &[DataRecord]) -> Vec<HashMap<String, f64>> {
    records.iter()
        .filter(|record| record.validate().is_ok())
        .map(|record| record.calculate_statistics())
        .collect()
}

pub fn filter_records_by_metadata(
    records: &[DataRecord],
    key: &str,
    value: &str
) -> Vec<DataRecord> {
    records.iter()
        .filter(|record| {
            record.metadata.get(key)
                .map(|v| v == value)
                .unwrap_or(false)
        })
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_validation() {
        let mut record = DataRecord::new(1, 1000);
        record.add_value(42.5);
        
        assert!(record.validate().is_ok());
        
        let invalid_record = DataRecord::new(0, 1000);
        assert!(invalid_record.validate().is_err());
    }

    #[test]
    fn test_statistics_calculation() {
        let mut record = DataRecord::new(1, 1000);
        record.add_value(10.0)
              .add_value(20.0)
              .add_value(30.0);
        
        let stats = record.calculate_statistics();
        
        assert_eq!(stats.get("mean"), Some(&20.0));
        assert_eq!(stats.get("sum"), Some(&60.0));
        assert_eq!(stats.get("count"), Some(&3.0));
    }

    #[test]
    fn test_metadata_filtering() {
        let mut record1 = DataRecord::new(1, 1000);
        record1.add_metadata("category", "A");
        
        let mut record2 = DataRecord::new(2, 2000);
        record2.add_metadata("category", "B");
        
        let records = vec![record1, record2];
        let filtered = filter_records_by_metadata(&records, "category", "A");
        
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, 1);
    }
}