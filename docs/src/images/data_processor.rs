
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DataRecord {
    pub id: u64,
    pub timestamp: i64,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, PartialEq)]
pub enum ValidationError {
    InvalidId,
    InvalidTimestamp,
    EmptyValues,
    MetadataTooLarge,
}

pub struct DataProcessor {
    max_metadata_size: usize,
    min_timestamp: i64,
}

impl DataProcessor {
    pub fn new(max_metadata_size: usize, min_timestamp: i64) -> Self {
        DataProcessor {
            max_metadata_size,
            min_timestamp,
        }
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), ValidationError> {
        if record.id == 0 {
            return Err(ValidationError::InvalidId);
        }

        if record.timestamp < self.min_timestamp {
            return Err(ValidationError::InvalidTimestamp);
        }

        if record.values.is_empty() {
            return Err(ValidationError::EmptyValues);
        }

        let total_metadata_size: usize = record
            .metadata
            .iter()
            .map(|(k, v)| k.len() + v.len())
            .sum();

        if total_metadata_size > self.max_metadata_size {
            return Err(ValidationError::MetadataTooLarge);
        }

        Ok(())
    }

    pub fn transform_values(&self, record: &mut DataRecord, multiplier: f64) {
        for value in record.values.iter_mut() {
            *value *= multiplier;
        }
    }

    pub fn calculate_statistics(&self, records: &[DataRecord]) -> HashMap<String, f64> {
        let mut stats = HashMap::new();

        if records.is_empty() {
            return stats;
        }

        let total_values: usize = records.iter().map(|r| r.values.len()).sum();
        let sum_all: f64 = records
            .iter()
            .flat_map(|r| r.values.iter())
            .sum();

        let avg = sum_all / total_values as f64;

        let variance: f64 = records
            .iter()
            .flat_map(|r| r.values.iter())
            .map(|v| (v - avg).powi(2))
            .sum::<f64>() / total_values as f64;

        stats.insert("record_count".to_string(), records.len() as f64);
        stats.insert("total_values".to_string(), total_values as f64);
        stats.insert("average".to_string(), avg);
        stats.insert("variance".to_string(), variance);
        stats.insert("std_deviation".to_string(), variance.sqrt());

        stats
    }

    pub fn filter_by_timestamp(
        &self,
        records: &[DataRecord],
        start: i64,
        end: i64,
    ) -> Vec<DataRecord> {
        records
            .iter()
            .filter(|r| r.timestamp >= start && r.timestamp <= end)
            .cloned()
            .collect()
    }
}

pub fn serialize_records(records: &[DataRecord]) -> Result<String, Box<dyn Error>> {
    let json = serde_json::to_string(records)?;
    Ok(json)
}

pub fn deserialize_records(json: &str) -> Result<Vec<DataRecord>, Box<dyn Error>> {
    let records: Vec<DataRecord> = serde_json::from_str(json)?;
    Ok(records)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation() {
        let processor = DataProcessor::new(100, 0);
        let mut record = DataRecord {
            id: 1,
            timestamp: 1000,
            values: vec![1.0, 2.0, 3.0],
            metadata: HashMap::new(),
        };

        assert!(processor.validate_record(&record).is_ok());

        record.id = 0;
        assert_eq!(
            processor.validate_record(&record).unwrap_err(),
            ValidationError::InvalidId
        );

        record.id = 1;
        record.timestamp = -1;
        assert_eq!(
            processor.validate_record(&record).unwrap_err(),
            ValidationError::InvalidTimestamp
        );

        record.timestamp = 1000;
        record.values.clear();
        assert_eq!(
            processor.validate_record(&record).unwrap_err(),
            ValidationError::EmptyValues
        );
    }

    #[test]
    fn test_value_transformation() {
        let processor = DataProcessor::new(100, 0);
        let mut record = DataRecord {
            id: 1,
            timestamp: 1000,
            values: vec![1.0, 2.0, 3.0],
            metadata: HashMap::new(),
        };

        processor.transform_values(&mut record, 2.0);
        assert_eq!(record.values, vec![2.0, 4.0, 6.0]);
    }

    #[test]
    fn test_statistics_calculation() {
        let processor = DataProcessor::new(100, 0);
        let records = vec![
            DataRecord {
                id: 1,
                timestamp: 1000,
                values: vec![1.0, 2.0],
                metadata: HashMap::new(),
            },
            DataRecord {
                id: 2,
                timestamp: 2000,
                values: vec![3.0, 4.0],
                metadata: HashMap::new(),
            },
        ];

        let stats = processor.calculate_statistics(&records);
        assert_eq!(stats.get("record_count"), Some(&2.0));
        assert_eq!(stats.get("total_values"), Some(&4.0));
        assert_eq!(stats.get("average"), Some(&2.5));
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct DataRecord {
    id: u32,
    value: f64,
    category: String,
    valid: bool,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: String) -> Self {
        let valid = value >= 0.0 && value <= 1000.0;
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

pub struct DataProcessor {
    records: Vec<DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<usize, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut count = 0;

        for (index, line) in reader.lines().enumerate() {
            if index == 0 {
                continue;
            }

            let line = line?;
            let parts: Vec<&str> = line.split(',').collect();
            
            if parts.len() >= 3 {
                let id = parts[0].parse::<u32>().unwrap_or(0);
                let value = parts[1].parse::<f64>().unwrap_or(0.0);
                let category = parts[2].to_string();
                
                let record = DataRecord::new(id, value, category);
                self.records.push(record);
                count += 1;
            }
        }

        Ok(count)
    }

    pub fn filter_valid(&self) -> Vec<&DataRecord> {
        self.records.iter().filter(|r| r.is_valid()).collect()
    }

    pub fn calculate_average(&self) -> f64 {
        let valid_records: Vec<&DataRecord> = self.filter_valid();
        if valid_records.is_empty() {
            return 0.0;
        }
        
        let sum: f64 = valid_records.iter().map(|r| r.value).sum();
        sum / valid_records.len() as f64
    }

    pub fn count_by_category(&self) -> std::collections::HashMap<String, usize> {
        let mut categories = std::collections::HashMap::new();
        
        for record in &self.records {
            if record.is_valid() {
                *categories.entry(record.category.clone()).or_insert(0) += 1;
            }
        }
        
        categories
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
        assert_eq!(record.id, 1);
        assert_eq!(record.value, 42.5);
        assert_eq!(record.category, "test");
        assert!(record.is_valid());
    }

    #[test]
    fn test_invalid_record() {
        let record = DataRecord::new(2, -10.0, "invalid".to_string());
        assert!(!record.is_valid());
    }

    #[test]
    fn test_csv_loading() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category").unwrap();
        writeln!(temp_file, "1,100.5,alpha").unwrap();
        writeln!(temp_file, "2,200.3,beta").unwrap();
        writeln!(temp_file, "3,-50.0,gamma").unwrap();

        let mut processor = DataProcessor::new();
        let result = processor.load_from_csv(temp_file.path());
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 3);
        assert_eq!(processor.filter_valid().len(), 2);
    }

    #[test]
    fn test_average_calculation() {
        let mut processor = DataProcessor::new();
        processor.records.push(DataRecord::new(1, 100.0, "test".to_string()));
        processor.records.push(DataRecord::new(2, 200.0, "test".to_string()));
        processor.records.push(DataRecord::new(3, -10.0, "test".to_string()));
        
        let average = processor.calculate_average();
        assert_eq!(average, 150.0);
    }
}