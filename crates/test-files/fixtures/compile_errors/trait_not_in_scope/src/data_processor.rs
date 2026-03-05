use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Serialize, Deserialize)]
pub struct DataRecord {
    pub id: u64,
    pub value: f64,
    pub timestamp: i64,
}

#[derive(Debug, Error)]
pub enum ProcessingError {
    #[error("Invalid data value: {0}")]
    InvalidValue(String),
    #[error("Timestamp out of range")]
    InvalidTimestamp,
    #[error("Serialization error")]
    SerializationFailed,
}

pub fn validate_record(record: &DataRecord) -> Result<(), ProcessingError> {
    if record.value.is_nan() || record.value.is_infinite() {
        return Err(ProcessingError::InvalidValue(
            "Value must be finite".to_string(),
        ));
    }

    if record.timestamp < 0 {
        return Err(ProcessingError::InvalidTimestamp);
    }

    Ok(())
}

pub fn transform_record(record: &DataRecord, multiplier: f64) -> DataRecord {
    DataRecord {
        id: record.id,
        value: record.value * multiplier,
        timestamp: record.timestamp,
    }
}

pub fn process_records(
    records: &[DataRecord],
    multiplier: f64,
) -> Result<Vec<DataRecord>, ProcessingError> {
    let mut processed = Vec::with_capacity(records.len());

    for record in records {
        validate_record(record)?;
        let transformed = transform_record(record, multiplier);
        processed.push(transformed);
    }

    Ok(processed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_valid_record() {
        let record = DataRecord {
            id: 1,
            value: 42.5,
            timestamp: 1234567890,
        };
        assert!(validate_record(&record).is_ok());
    }

    #[test]
    fn test_validate_invalid_value() {
        let record = DataRecord {
            id: 1,
            value: f64::INFINITY,
            timestamp: 1234567890,
        };
        assert!(validate_record(&record).is_err());
    }

    #[test]
    fn test_transform_record() {
        let record = DataRecord {
            id: 1,
            value: 10.0,
            timestamp: 1000,
        };
        let transformed = transform_record(&record, 2.5);
        assert_eq!(transformed.value, 25.0);
        assert_eq!(transformed.id, record.id);
    }

    #[test]
    fn test_process_records() {
        let records = vec![
            DataRecord {
                id: 1,
                value: 2.0,
                timestamp: 1000,
            },
            DataRecord {
                id: 2,
                value: 3.0,
                timestamp: 2000,
            },
        ];

        let result = process_records(&records, 3.0).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].value, 6.0);
        assert_eq!(result[1].value, 9.0);
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
    let mut rdr = Reader::from_reader(file);
    let mut records = Vec::new();

    for result in rdr.deserialize() {
        let record: Record = result?;
        validate_record(&record)?;
        records.push(record);
    }

    Ok(records)
}

fn validate_record(record: &Record) -> Result<(), Box<dyn Error>> {
    if record.name.is_empty() {
        return Err("Name cannot be empty".into());
    }
    if record.value < 0.0 {
        return Err("Value cannot be negative".into());
    }
    if !["A", "B", "C"].contains(&record.category.as_str()) {
        return Err("Invalid category".into());
    }
    Ok(())
}

pub fn calculate_statistics(records: &[Record]) -> (f64, f64, f64) {
    let count = records.len() as f64;
    if count == 0.0 {
        return (0.0, 0.0, 0.0);
    }

    let sum: f64 = records.iter().map(|r| r.value).sum();
    let mean = sum / count;
    
    let variance: f64 = records.iter()
        .map(|r| (r.value - mean).powi(2))
        .sum::<f64>() / count;
    
    let std_dev = variance.sqrt();
    
    (sum, mean, std_dev)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_valid_data_processing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,category").unwrap();
        writeln!(temp_file, "1,Test1,10.5,A").unwrap();
        writeln!(temp_file, "2,Test2,20.0,B").unwrap();
        
        let records = process_data_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].name, "Test1");
    }

    #[test]
    fn test_statistics_calculation() {
        let records = vec![
            Record { id: 1, name: "A".to_string(), value: 10.0, category: "A".to_string() },
            Record { id: 2, name: "B".to_string(), value: 20.0, category: "B".to_string() },
            Record { id: 3, name: "C".to_string(), value: 30.0, category: "C".to_string() },
        ];
        
        let (sum, mean, std_dev) = calculate_statistics(&records);
        assert_eq!(sum, 60.0);
        assert_eq!(mean, 20.0);
        assert!(std_dev > 8.16 && std_dev < 8.17);
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct DataProcessor {
    data: Vec<f64>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor { data: Vec::new() }
    }

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        
        for line in reader.lines() {
            let line = line?;
            if let Ok(value) = line.trim().parse::<f64>() {
                self.data.push(value);
            }
        }
        
        Ok(())
    }

    pub fn calculate_mean(&self) -> Option<f64> {
        if self.data.is_empty() {
            return None;
        }
        
        let sum: f64 = self.data.iter().sum();
        Some(sum / self.data.len() as f64)
    }

    pub fn calculate_standard_deviation(&self) -> Option<f64> {
        if self.data.len() < 2 {
            return None;
        }
        
        let mean = self.calculate_mean()?;
        let variance: f64 = self.data
            .iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / (self.data.len() - 1) as f64;
        
        Some(variance.sqrt())
    }

    pub fn filter_outliers(&self, threshold: f64) -> Vec<f64> {
        if let Some(std_dev) = self.calculate_standard_deviation() {
            if let Some(mean) = self.calculate_mean() {
                let lower_bound = mean - threshold * std_dev;
                let upper_bound = mean + threshold * std_dev;
                
                return self.data
                    .iter()
                    .filter(|&&x| x >= lower_bound && x <= upper_bound)
                    .cloned()
                    .collect();
            }
        }
        
        self.data.clone()
    }

    pub fn get_summary(&self) -> String {
        let mean = self.calculate_mean().unwrap_or(0.0);
        let std_dev = self.calculate_standard_deviation().unwrap_or(0.0);
        let min = self.data.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = self.data.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        
        format!(
            "Data points: {}, Mean: {:.4}, Std Dev: {:.4}, Range: [{:.4}, {:.4}]",
            self.data.len(),
            mean,
            std_dev,
            min,
            max
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processing() {
        let mut processor = DataProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "10.5\n15.2\n12.8\n14.1\n11.9").unwrap();
        
        processor.load_from_csv(temp_file.path()).unwrap();
        
        assert_eq!(processor.data.len(), 5);
        assert!(processor.calculate_mean().unwrap() > 0.0);
        assert!(processor.calculate_standard_deviation().unwrap() > 0.0);
        
        let filtered = processor.filter_outliers(2.0);
        assert!(filtered.len() <= 5);
        
        let summary = processor.get_summary();
        assert!(summary.contains("Data points: 5"));
    }
}