
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
            return Err("Empty values array".into());
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
            record.add_metadata("processed".to_string(), "true".to_string());
            record
        })
        .collect()
}

pub fn calculate_statistics(records: &[DataRecord]) -> HashMap<String, f64> {
    let mut stats = HashMap::new();
    
    if records.is_empty() {
        return stats;
    }

    let total_values: usize = records.iter().map(|r| r.values.len()).sum();
    let sum_all: f64 = records.iter().flat_map(|r| &r.values).sum();
    let count_all = total_values as f64;

    stats.insert("mean".to_string(), sum_all / count_all);
    stats.insert("total_records".to_string(), records.len() as f64);
    stats.insert("total_values".to_string(), total_values as f64);

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
        assert_eq!(processed[1].metadata.get("processed"), Some(&"true".to_string()));
    }

    #[test]
    fn test_calculate_statistics() {
        let records = vec![
            DataRecord::new(1, 1000, vec![1.0, 2.0]),
            DataRecord::new(2, 2000, vec![3.0, 4.0]),
        ];
        
        let stats = calculate_statistics(&records);
        assert_eq!(stats.get("mean"), Some(&2.5));
        assert_eq!(stats.get("total_records"), Some(&2.0));
    }
}
use csv::Reader;
use serde::Deserialize;
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

pub fn process_data_file(file_path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let mut reader = Reader::from_reader(file);
    
    let mut records = Vec::new();
    for result in reader.deserialize() {
        let record: Record = result?;
        validate_record(&record)?;
        records.push(record);
    }
    
    Ok(records)
}

fn validate_record(record: &Record) -> Result<(), String> {
    if record.id == 0 {
        return Err("ID cannot be zero".to_string());
    }
    
    if record.name.is_empty() {
        return Err("Name cannot be empty".to_string());
    }
    
    if record.value < 0.0 {
        return Err("Value cannot be negative".to_string());
    }
    
    let valid_categories = ["A", "B", "C", "D"];
    if !valid_categories.contains(&record.category.as_str()) {
        return Err(format!("Invalid category: {}", record.category));
    }
    
    Ok(())
}

pub fn calculate_statistics(records: &[Record]) -> (f64, f64, f64) {
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
    use tempfile::NamedTempFile;
    use std::io::Write;
    
    #[test]
    fn test_process_valid_data() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,category").unwrap();
        writeln!(temp_file, "1,Test1,100.5,A").unwrap();
        writeln!(temp_file, "2,Test2,200.0,B").unwrap();
        
        let result = process_data_file(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 2);
    }
    
    #[test]
    fn test_validate_record() {
        let valid_record = Record {
            id: 1,
            name: "Test".to_string(),
            value: 50.0,
            category: "C".to_string(),
        };
        assert!(validate_record(&valid_record).is_ok());
        
        let invalid_record = Record {
            id: 0,
            name: "".to_string(),
            value: -10.0,
            category: "X".to_string(),
        };
        assert!(validate_record(&invalid_record).is_err());
    }
    
    #[test]
    fn test_calculate_statistics() {
        let records = vec![
            Record { id: 1, name: "A".to_string(), value: 10.0, category: "A".to_string() },
            Record { id: 2, name: "B".to_string(), value: 20.0, category: "B".to_string() },
            Record { id: 3, name: "C".to_string(), value: 30.0, category: "C".to_string() },
        ];
        
        let (mean, variance, std_dev) = calculate_statistics(&records);
        assert_eq!(mean, 20.0);
        assert_eq!(variance, 66.66666666666667);
        assert_eq!(std_dev, 8.16496580927726);
    }
}