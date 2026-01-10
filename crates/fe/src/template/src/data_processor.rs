
use std::error::Error;
use std::fs::File;
use std::path::Path;

#[derive(Debug)]
pub struct DataRecord {
    id: u32,
    value: f64,
    category: String,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: String) -> Result<Self, String> {
        if value < 0.0 {
            return Err("Value cannot be negative".to_string());
        }
        if category.is_empty() {
            return Err("Category cannot be empty".to_string());
        }
        Ok(Self { id, value, category })
    }

    pub fn calculate_adjusted_value(&self, multiplier: f64) -> f64 {
        self.value * multiplier
    }
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        Self { records: Vec::new() }
    }

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
        let mut rdr = csv::Reader::from_reader(file);

        for result in rdr.records() {
            let record = result?;
            if record.len() != 3 {
                continue;
            }

            let id: u32 = match record[0].parse() {
                Ok(val) => val,
                Err(_) => continue,
            };

            let value: f64 = match record[1].parse() {
                Ok(val) => val,
                Err(_) => continue,
            };

            let category = record[2].to_string();

            match DataRecord::new(id, value, category) {
                Ok(data_record) => self.records.push(data_record),
                Err(_) => continue,
            }
        }

        Ok(())
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    pub fn calculate_total_value(&self) -> f64 {
        self.records.iter().map(|record| record.value).sum()
    }

    pub fn get_statistics(&self) -> (f64, f64, f64) {
        if self.records.is_empty() {
            return (0.0, 0.0, 0.0);
        }

        let count = self.records.len() as f64;
        let total: f64 = self.records.iter().map(|r| r.value).sum();
        let mean = total / count;

        let variance: f64 = self.records
            .iter()
            .map(|r| (r.value - mean).powi(2))
            .sum::<f64>() / count;

        let std_dev = variance.sqrt();

        (mean, variance, std_dev)
    }

    pub fn record_count(&self) -> usize {
        self.records.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_record_creation() {
        let record = DataRecord::new(1, 42.5, "test".to_string());
        assert!(record.is_ok());

        let record = DataRecord::new(2, -10.0, "test".to_string());
        assert!(record.is_err());

        let record = DataRecord::new(3, 15.0, "".to_string());
        assert!(record.is_err());
    }

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        assert_eq!(processor.record_count(), 0);

        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category").unwrap();
        writeln!(temp_file, "1,10.5,category_a").unwrap();
        writeln!(temp_file, "2,20.0,category_b").unwrap();
        writeln!(temp_file, "3,15.75,category_a").unwrap();

        let result = processor.load_from_csv(temp_file.path());
        assert!(result.is_ok());
        assert_eq!(processor.record_count(), 3);

        let filtered = processor.filter_by_category("category_a");
        assert_eq!(filtered.len(), 2);

        let total = processor.calculate_total_value();
        assert!((total - 46.25).abs() < 0.001);

        let stats = processor.get_statistics();
        assert!((stats.0 - 15.416666).abs() < 0.001);
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
    if record.name.is_empty() {
        return Err("Name cannot be empty".to_string());
    }
    if record.value < 0.0 {
        return Err("Value must be non-negative".to_string());
    }
    if !["A", "B", "C"].contains(&record.category.as_str()) {
        return Err("Category must be A, B, or C".to_string());
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
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_process_valid_data() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,category").unwrap();
        writeln!(temp_file, "1,Item1,10.5,A").unwrap();
        writeln!(temp_file, "2,Item2,20.0,B").unwrap();
        
        let result = process_data_file(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 2);
    }

    #[test]
    fn test_statistics_calculation() {
        let records = vec![
            Record { id: 1, name: "Test1".to_string(), value: 10.0, category: "A".to_string() },
            Record { id: 2, name: "Test2".to_string(), value: 20.0, category: "B".to_string() },
            Record { id: 3, name: "Test3".to_string(), value: 30.0, category: "C".to_string() },
        ];
        
        let (sum, mean, std_dev) = calculate_statistics(&records);
        assert_eq!(sum, 60.0);
        assert_eq!(mean, 20.0);
        assert!(std_dev > 8.16 && std_dev < 8.17);
    }
}
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRecord {
    pub id: u64,
    pub timestamp: i64,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

#[derive(Error, Debug)]
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
        if record.id == 0 {
            return Err(ProcessingError::ValidationFailed("ID cannot be zero".to_string()));
        }

        if record.values.is_empty() {
            return Err(ProcessingError::ValidationFailed("Values cannot be empty".to_string()));
        }

        for value in &record.values {
            if value.abs() > self.validation_threshold {
                return Err(ProcessingError::ValidationFailed(
                    format!("Value {} exceeds threshold {}", value, self.validation_threshold)
                ));
            }
        }

        Ok(())
    }

    pub fn transform_record(&self, record: &DataRecord) -> Result<DataRecord, ProcessingError> {
        let mut transformed = record.clone();
        
        transformed.values = record.values
            .iter()
            .map(|v| v * self.normalization_factor)
            .collect();

        transformed.metadata.insert(
            "processed_timestamp".to_string(),
            chrono::Utc::now().timestamp().to_string()
        );

        transformed.metadata.insert(
            "normalization_factor".to_string(),
            self.normalization_factor.to_string()
        );

        Ok(transformed)
    }

    pub fn process_batch(&self, records: Vec<DataRecord>) -> Result<Vec<DataRecord>, ProcessingError> {
        let mut processed_records = Vec::with_capacity(records.len());

        for record in records {
            self.validate_record(&record)?;
            let transformed = self.transform_record(&record)?;
            processed_records.push(transformed);
        }

        Ok(processed_records)
    }

    pub fn calculate_statistics(&self, records: &[DataRecord]) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        
        if records.is_empty() {
            return stats;
        }

        let total_values: usize = records.iter().map(|r| r.values.len()).sum();
        let sum_all: f64 = records.iter()
            .flat_map(|r| r.values.iter())
            .sum();

        let avg = sum_all / total_values as f64;

        let variance: f64 = records.iter()
            .flat_map(|r| r.values.iter())
            .map(|v| (v - avg).powi(2))
            .sum::<f64>() / total_values as f64;

        stats.insert("record_count".to_string(), records.len() as f64);
        stats.insert("total_values".to_string(), total_values as f64);
        stats.insert("average".to_string(), avg);
        stats.insert("variance".to_string(), variance);
        stats.insert("standard_deviation".to_string(), variance.sqrt());

        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_record() -> DataRecord {
        DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: vec![1.0, 2.0, 3.0],
            metadata: HashMap::from([
                ("source".to_string(), "test".to_string())
            ]),
        }
    }

    #[test]
    fn test_validation_success() {
        let processor = DataProcessor::new(100.0, 1.0);
        let record = create_test_record();
        assert!(processor.validate_record(&record).is_ok());
    }

    #[test]
    fn test_validation_failure() {
        let processor = DataProcessor::new(2.0, 1.0);
        let record = create_test_record();
        assert!(processor.validate_record(&record).is_err());
    }

    #[test]
    fn test_transform_record() {
        let processor = DataProcessor::new(100.0, 2.0);
        let record = create_test_record();
        let transformed = processor.transform_record(&record).unwrap();
        
        assert_eq!(transformed.values, vec![2.0, 4.0, 6.0]);
        assert!(transformed.metadata.contains_key("processed_timestamp"));
        assert!(transformed.metadata.contains_key("normalization_factor"));
    }

    #[test]
    fn test_process_batch() {
        let processor = DataProcessor::new(100.0, 1.5);
        let records = vec![create_test_record(), create_test_record()];
        let processed = processor.process_batch(records).unwrap();
        
        assert_eq!(processed.len(), 2);
        assert_eq!(processed[0].values, vec![1.5, 3.0, 4.5]);
    }

    #[test]
    fn test_calculate_statistics() {
        let processor = DataProcessor::new(100.0, 1.0);
        let records = vec![
            DataRecord {
                id: 1,
                timestamp: 1,
                values: vec![1.0, 2.0],
                metadata: HashMap::new(),
            },
            DataRecord {
                id: 2,
                timestamp: 2,
                values: vec![3.0, 4.0],
                metadata: HashMap::new(),
            },
        ];
        
        let stats = processor.calculate_statistics(&records);
        
        assert_eq!(stats["record_count"], 2.0);
        assert_eq!(stats["total_values"], 4.0);
        assert_eq!(stats["average"], 2.5);
    }
}