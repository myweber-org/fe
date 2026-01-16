
use std::collections::HashMap;
use std::error::Error;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub timestamp: i64,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

impl DataRecord {
    pub fn new(id: u32, timestamp: i64, values: Vec<f64>) -> Self {
        Self {
            id,
            timestamp,
            values,
            metadata: HashMap::new(),
        }
    }

    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    pub fn validate(&self) -> Result<(), Box<dyn Error>> {
        if self.id == 0 {
            return Err("Invalid record ID".into());
        }
        if self.timestamp < 0 {
            return Err("Invalid timestamp".into());
        }
        if self.values.is_empty() {
            return Err("Empty values vector".into());
        }
        Ok(())
    }
}

pub fn process_records(records: Vec<DataRecord>) -> Vec<DataRecord> {
    records
        .into_iter()
        .filter(|record| record.validate().is_ok())
        .map(|mut record| {
            let transformed_values: Vec<f64> = record
                .values
                .iter()
                .map(|&value| value * 2.0)
                .collect();
            record.values = transformed_values;
            record
        })
        .collect()
}

pub fn calculate_statistics(records: &[DataRecord]) -> HashMap<String, f64> {
    let mut stats = HashMap::new();
    
    if records.is_empty() {
        return stats;
    }

    let total_records = records.len() as f64;
    let mut sum_values = 0.0;
    let mut count_values = 0;

    for record in records {
        for &value in &record.values {
            sum_values += value;
            count_values += 1;
        }
    }

    if count_values > 0 {
        let average = sum_values / count_values as f64;
        stats.insert("average".to_string(), average);
        stats.insert("total_records".to_string(), total_records);
        stats.insert("total_values".to_string(), count_values as f64);
    }

    stats
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_validation() {
        let valid_record = DataRecord::new(1, 1234567890, vec![1.0, 2.0, 3.0]);
        assert!(valid_record.validate().is_ok());

        let invalid_record = DataRecord::new(0, 1234567890, vec![1.0]);
        assert!(invalid_record.validate().is_err());
    }

    #[test]
    fn test_process_records() {
        let records = vec![
            DataRecord::new(1, 1000, vec![1.0, 2.0]),
            DataRecord::new(2, 2000, vec![3.0, 4.0]),
        ];
        
        let processed = process_records(records);
        assert_eq!(processed.len(), 2);
        assert_eq!(processed[0].values, vec![2.0, 4.0]);
        assert_eq!(processed[1].values, vec![6.0, 8.0]);
    }

    #[test]
    fn test_calculate_statistics() {
        let records = vec![
            DataRecord::new(1, 1000, vec![1.0, 2.0]),
            DataRecord::new(2, 2000, vec![3.0, 4.0]),
        ];
        
        let stats = calculate_statistics(&records);
        assert_eq!(stats["average"], 2.5);
        assert_eq!(stats["total_records"], 2.0);
        assert_eq!(stats["total_values"], 4.0);
    }
}
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u64,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

impl DataRecord {
    pub fn new(id: u64, values: Vec<f64>) -> Self {
        Self {
            id,
            values,
            metadata: HashMap::new(),
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.id == 0 {
            return Err("Invalid record ID".to_string());
        }

        if self.values.is_empty() {
            return Err("Empty values array".to_string());
        }

        for value in &self.values {
            if value.is_nan() || value.is_infinite() {
                return Err("Invalid numeric value detected".to_string());
            }
        }

        Ok(())
    }

    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }
}

pub fn normalize_values(values: &[f64]) -> Vec<f64> {
    if values.is_empty() {
        return Vec::new();
    }

    let max_value = values
        .iter()
        .fold(f64::NEG_INFINITY, |a, &b| a.max(b));

    if max_value <= 0.0 {
        return values.to_vec();
    }

    values
        .iter()
        .map(|&v| v / max_value)
        .collect()
}

pub fn process_records(records: &[DataRecord]) -> Vec<DataRecord> {
    records
        .iter()
        .filter(|record| record.validate().is_ok())
        .map(|record| {
            let mut processed = record.clone();
            processed.values = normalize_values(&record.values);
            processed
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_validation() {
        let valid_record = DataRecord::new(1, vec![1.0, 2.0, 3.0]);
        assert!(valid_record.validate().is_ok());

        let invalid_record = DataRecord::new(0, vec![1.0, 2.0]);
        assert!(invalid_record.validate().is_err());
    }

    #[test]
    fn test_normalize_values() {
        let values = vec![1.0, 2.0, 3.0, 4.0];
        let normalized = normalize_values(&values);
        assert_eq!(normalized, vec![0.25, 0.5, 0.75, 1.0]);
    }

    #[test]
    fn test_metadata_operations() {
        let mut record = DataRecord::new(1, vec![1.0]);
        record.add_metadata("source".to_string(), "test".to_string());
        
        assert_eq!(record.get_metadata("source"), Some(&"test".to_string()));
        assert_eq!(record.get_metadata("nonexistent"), None);
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub category: String,
}

pub fn process_csv_file(file_path: &str) -> Result<Vec<DataRecord>, Box<dyn Error>> {
    let path = Path::new(file_path);
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    
    let mut records = Vec::new();
    let mut line_number = 0;
    
    for line in reader.lines() {
        line_number += 1;
        let line_content = line?;
        
        if line_content.trim().is_empty() || line_content.starts_with('#') {
            continue;
        }
        
        let fields: Vec<&str> = line_content.split(',').collect();
        
        if fields.len() != 3 {
            return Err(format!("Invalid field count at line {}", line_number).into());
        }
        
        let id = fields[0].trim().parse::<u32>()
            .map_err(|e| format!("Invalid ID at line {}: {}", line_number, e))?;
        
        let value = fields[1].trim().parse::<f64>()
            .map_err(|e| format!("Invalid value at line {}: {}", line_number, e))?;
        
        let category = fields[2].trim().to_string();
        
        if category.is_empty() {
            return Err(format!("Empty category at line {}", line_number).into());
        }
        
        records.push(DataRecord { id, value, category });
    }
    
    if records.is_empty() {
        return Err("No valid records found in file".into());
    }
    
    Ok(records)
}

pub fn calculate_statistics(records: &[DataRecord]) -> (f64, f64, f64) {
    if records.is_empty() {
        return (0.0, 0.0, 0.0);
    }
    
    let sum: f64 = records.iter().map(|r| r.value).sum();
    let count = records.len() as f64;
    let mean = sum / count;
    
    let variance: f64 = records.iter()
        .map(|r| (r.value - mean).powi(2))
        .sum::<f64>() / count;
    
    let std_dev = variance.sqrt();
    
    (mean, variance, std_dev)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_valid_csv_processing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "1,45.5,CategoryA").unwrap();
        writeln!(temp_file, "2,67.8,CategoryB").unwrap();
        writeln!(temp_file, "3,89.2,CategoryA").unwrap();
        
        let records = process_csv_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(records.len(), 3);
        assert_eq!(records[0].id, 1);
        assert_eq!(records[1].category, "CategoryB");
    }
    
    #[test]
    fn test_statistics_calculation() {
        let records = vec![
            DataRecord { id: 1, value: 10.0, category: "Test".to_string() },
            DataRecord { id: 2, value: 20.0, category: "Test".to_string() },
            DataRecord { id: 3, value: 30.0, category: "Test".to_string() },
        ];
        
        let (mean, variance, std_dev) = calculate_statistics(&records);
        assert_eq!(mean, 20.0);
        assert_eq!(variance, 66.66666666666667);
        assert_eq!(std_dev, 8.16496580927726);
    }
}