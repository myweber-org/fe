
use csv::Reader;
use serde::Deserialize;
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    active: bool,
}

pub fn process_data_file(file_path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let mut rdr = Reader::from_reader(file);
    let mut records = Vec::new();

    for result in rdr.deserialize() {
        let record: Record = result?;
        if record.value >= 0.0 {
            records.push(record);
        }
    }

    Ok(records)
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

    (mean, variance, std_dev)
}

pub fn filter_active_records(records: Vec<Record>) -> Vec<Record> {
    records.into_iter()
        .filter(|r| r.active)
        .collect()
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
    active: bool,
}

pub fn process_data_file(path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(path)?;
    let mut rdr = Reader::from_reader(file);
    let mut records = Vec::new();

    for result in rdr.deserialize() {
        let record: Record = result?;
        if record.value >= 0.0 {
            records.push(record);
        }
    }

    Ok(records)
}

pub fn calculate_statistics(records: &[Record]) -> (f64, f64, usize) {
    let count = records.len();
    let sum: f64 = records.iter().map(|r| r.value).sum();
    let avg = if count > 0 { sum / count as f64 } else { 0.0 };
    
    let variance: f64 = records.iter()
        .map(|r| (r.value - avg).powi(2))
        .sum::<f64>() / count as f64;
    
    (avg, variance.sqrt(), count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_process_valid_data() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,active").unwrap();
        writeln!(temp_file, "1,Test1,10.5,true").unwrap();
        writeln!(temp_file, "2,Test2,-3.0,false").unwrap();
        writeln!(temp_file, "3,Test3,7.2,true").unwrap();

        let records = process_data_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].name, "Test1");
        assert_eq!(records[1].name, "Test3");
    }

    #[test]
    fn test_calculate_statistics() {
        let records = vec![
            Record { id: 1, name: "A".to_string(), value: 10.0, active: true },
            Record { id: 2, name: "B".to_string(), value: 20.0, active: false },
            Record { id: 3, name: "C".to_string(), value: 30.0, active: true },
        ];

        let (avg, std_dev, count) = calculate_statistics(&records);
        assert_eq!(avg, 20.0);
        assert_eq!(count, 3);
        assert!(std_dev - 8.164965 < 0.0001);
    }
}use csv::Reader;
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
        if record.value >= 0.0 {
            records.push(record);
        }
    }

    Ok(records)
}

pub fn calculate_statistics(records: &[Record]) -> (f64, f64, usize) {
    let sum: f64 = records.iter().map(|r| r.value).sum();
    let count = records.len();
    let average = if count > 0 { sum / count as f64 } else { 0.0 };
    
    (sum, average, count)
}

pub fn filter_by_category(records: Vec<Record>, category: &str) -> Vec<Record> {
    records.into_iter()
        .filter(|r| r.category == category)
        .collect()
}
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DataRecord {
    pub id: u64,
    pub timestamp: i64,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Error)]
pub enum ProcessingError {
    #[error("Invalid data format")]
    InvalidFormat,
    #[error("Data validation failed: {0}")]
    ValidationFailed(String),
    #[error("Transformation error: {0}")]
    TransformationError(String),
}

pub struct DataProcessor {
    validation_threshold: f64,
    normalization_factor: f64,
}

impl DataProcessor {
    pub fn new(validation_threshold: f64, normalization_factor: f64) -> Self {
        Self {
            validation_threshold,
            normalization_factor,
        }
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if record.values.is_empty() {
            return Err(ProcessingError::ValidationFailed(
                "Empty values array".to_string(),
            ));
        }

        for value in &record.values {
            if value.is_nan() || value.is_infinite() {
                return Err(ProcessingError::ValidationFailed(
                    "Invalid numeric value".to_string(),
                ));
            }

            if value.abs() > self.validation_threshold {
                return Err(ProcessingError::ValidationFailed(format!(
                    "Value {} exceeds threshold {}",
                    value, self.validation_threshold
                )));
            }
        }

        Ok(())
    }

    pub fn normalize_values(&self, record: &mut DataRecord) -> Result<(), ProcessingError> {
        if self.normalization_factor == 0.0 {
            return Err(ProcessingError::TransformationError(
                "Normalization factor cannot be zero".to_string(),
            ));
        }

        for value in record.values.iter_mut() {
            *value /= self.normalization_factor;
        }

        Ok(())
    }

    pub fn calculate_statistics(&self, records: &[DataRecord]) -> HashMap<String, f64> {
        let mut stats = HashMap::new();

        if records.is_empty() {
            return stats;
        }

        let total_values: usize = records.iter().map(|r| r.values.len()).sum();
        stats.insert("total_records".to_string(), records.len() as f64);
        stats.insert("total_values".to_string(), total_values as f64);

        let all_values: Vec<f64> = records
            .iter()
            .flat_map(|r| r.values.iter().copied())
            .collect();

        if !all_values.is_empty() {
            let sum: f64 = all_values.iter().sum();
            let mean = sum / all_values.len() as f64;
            let variance: f64 = all_values.iter().map(|v| (v - mean).powi(2)).sum::<f64>()
                / all_values.len() as f64;
            let std_dev = variance.sqrt();

            stats.insert("mean".to_string(), mean);
            stats.insert("variance".to_string(), variance);
            stats.insert("std_dev".to_string(), std_dev);
            stats.insert(
                "min".to_string(),
                all_values.iter().fold(f64::INFINITY, |a, &b| a.min(b)),
            );
            stats.insert(
                "max".to_string(),
                all_values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b)),
            );
        }

        stats
    }

    pub fn process_batch(
        &self,
        records: Vec<DataRecord>,
    ) -> Result<(Vec<DataRecord>, HashMap<String, f64>), ProcessingError> {
        let mut processed_records = Vec::with_capacity(records.len());
        let mut validation_errors = 0;

        for mut record in records {
            match self.validate_record(&record) {
                Ok(_) => {
                    if let Err(e) = self.normalize_values(&mut record) {
                        return Err(e);
                    }
                    processed_records.push(record);
                }
                Err(_) => {
                    validation_errors += 1;
                }
            }
        }

        let stats = self.calculate_statistics(&processed_records);
        stats.insert("validation_errors".to_string(), validation_errors as f64);

        Ok((processed_records, stats))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_success() {
        let processor = DataProcessor::new(1000.0, 1.0);
        let record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: vec![10.5, 20.3, 30.7],
            metadata: HashMap::new(),
        };

        assert!(processor.validate_record(&record).is_ok());
    }

    #[test]
    fn test_validation_threshold_exceeded() {
        let processor = DataProcessor::new(10.0, 1.0);
        let record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: vec![5.0, 15.0, 8.0],
            metadata: HashMap::new(),
        };

        assert!(processor.validate_record(&record).is_err());
    }

    #[test]
    fn test_normalization() {
        let mut processor = DataProcessor::new(1000.0, 10.0);
        let mut record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: vec![100.0, 200.0, 300.0],
            metadata: HashMap::new(),
        };

        assert!(processor.normalize_values(&mut record).is_ok());
        assert_eq!(record.values, vec![10.0, 20.0, 30.0]);
    }

    #[test]
    fn test_statistics_calculation() {
        let processor = DataProcessor::new(1000.0, 1.0);
        let records = vec![
            DataRecord {
                id: 1,
                timestamp: 1234567890,
                values: vec![1.0, 2.0, 3.0],
                metadata: HashMap::new(),
            },
            DataRecord {
                id: 2,
                timestamp: 1234567891,
                values: vec![4.0, 5.0, 6.0],
                metadata: HashMap::new(),
            },
        ];

        let stats = processor.calculate_statistics(&records);
        assert_eq!(stats["total_records"], 2.0);
        assert_eq!(stats["total_values"], 6.0);
        assert_eq!(stats["mean"], 3.5);
    }
}