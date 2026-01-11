
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
        .filter_map(|mut record| {
            if record.validate().is_ok() {
                record.values = record.values.iter().map(|&v| v * 2.0).collect();
                Some(record)
            } else {
                None
            }
        })
        .collect()
}

pub fn calculate_statistics(records: &[DataRecord]) -> HashMap<String, f64> {
    let mut stats = HashMap::new();
    
    if records.is_empty() {
        return stats;
    }

    let total_values: Vec<f64> = records.iter().flat_map(|r| r.values.clone()).collect();
    
    if !total_values.is_empty() {
        let sum: f64 = total_values.iter().sum();
        let count = total_values.len() as f64;
        let mean = sum / count;
        
        let variance: f64 = total_values.iter()
            .map(|&value| (value - mean).powi(2))
            .sum::<f64>() / count;
        
        stats.insert("mean".to_string(), mean);
        stats.insert("variance".to_string(), variance);
        stats.insert("count".to_string(), count);
        stats.insert("sum".to_string(), sum);
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

        let invalid_record = DataRecord::new(0, 1234567890, vec![1.0, 2.0]);
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
        assert_eq!(stats["mean"], 2.5);
        assert_eq!(stats["count"], 4.0);
        assert_eq!(stats["sum"], 10.0);
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
    transformation_factor: f64,
}

impl DataProcessor {
    pub fn new(validation_threshold: f64, transformation_factor: f64) -> Self {
        Self {
            validation_threshold,
            transformation_factor,
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

    pub fn transform_values(&self, record: &mut DataRecord) -> Result<(), ProcessingError> {
        if record.values.iter().any(|v| v.is_nan()) {
            return Err(ProcessingError::TransformationError(
                "Cannot transform NaN values".to_string(),
            ));
        }

        for value in record.values.iter_mut() {
            *value *= self.transformation_factor;
        }

        record.metadata.insert(
            "transformed".to_string(),
            format!("factor_{}", self.transformation_factor),
        );

        Ok(())
    }

    pub fn process_batch(
        &self,
        records: Vec<DataRecord>,
    ) -> Result<Vec<DataRecord>, ProcessingError> {
        let mut processed_records = Vec::with_capacity(records.len());

        for mut record in records {
            self.validate_record(&record)?;
            self.transform_values(&mut record)?;
            processed_records.push(record);
        }

        Ok(processed_records)
    }

    pub fn calculate_statistics(&self, records: &[DataRecord]) -> HashMap<String, f64> {
        let mut stats = HashMap::new();

        if records.is_empty() {
            return stats;
        }

        let total_values: usize = records.iter().map(|r| r.values.len()).sum();
        let all_values: Vec<f64> = records
            .iter()
            .flat_map(|r| r.values.clone())
            .collect();

        if !all_values.is_empty() {
            let sum: f64 = all_values.iter().sum();
            let count = all_values.len() as f64;
            let mean = sum / count;

            let variance: f64 = all_values
                .iter()
                .map(|v| (v - mean).powi(2))
                .sum::<f64>()
                / count;

            stats.insert("mean".to_string(), mean);
            stats.insert("variance".to_string(), variance);
            stats.insert("total_records".to_string(), records.len() as f64);
            stats.insert("total_values".to_string(), total_values as f64);
        }

        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_success() {
        let processor = DataProcessor::new(1000.0, 2.0);
        let record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: vec![10.0, 20.0, 30.0],
            metadata: HashMap::new(),
        };

        assert!(processor.validate_record(&record).is_ok());
    }

    #[test]
    fn test_validation_threshold_exceeded() {
        let processor = DataProcessor::new(50.0, 2.0);
        let record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: vec![10.0, 60.0, 30.0],
            metadata: HashMap::new(),
        };

        assert!(processor.validate_record(&record).is_err());
    }

    #[test]
    fn test_transform_values() {
        let processor = DataProcessor::new(1000.0, 3.0);
        let mut record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: vec![1.0, 2.0, 3.0],
            metadata: HashMap::new(),
        };

        assert!(processor.transform_values(&mut record).is_ok());
        assert_eq!(record.values, vec![3.0, 6.0, 9.0]);
        assert_eq!(
            record.metadata.get("transformed"),
            Some(&"factor_3".to_string())
        );
    }

    #[test]
    fn test_process_batch() {
        let processor = DataProcessor::new(100.0, 2.0);
        let records = vec![
            DataRecord {
                id: 1,
                timestamp: 1234567890,
                values: vec![10.0, 20.0],
                metadata: HashMap::new(),
            },
            DataRecord {
                id: 2,
                timestamp: 1234567891,
                values: vec![30.0, 40.0],
                metadata: HashMap::new(),
            },
        ];

        let result = processor.process_batch(records);
        assert!(result.is_ok());
        let processed = result.unwrap();
        assert_eq!(processed.len(), 2);
        assert_eq!(processed[0].values, vec![20.0, 40.0]);
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct DataRecord {
    id: u32,
    value: f64,
    category: String,
    timestamp: String,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: String, timestamp: String) -> Self {
        DataRecord {
            id,
            value,
            category,
            timestamp,
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.category.is_empty() && self.value >= 0.0 && self.id > 0
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

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<usize, Box<dyn Error>> {
        let path = Path::new(file_path);
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        let mut count = 0;
        for (index, line) in reader.lines().enumerate() {
            let line = line?;
            
            if index == 0 {
                continue;
            }

            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() >= 4 {
                let id = parts[0].parse::<u32>().unwrap_or(0);
                let value = parts[1].parse::<f64>().unwrap_or(0.0);
                let category = parts[2].to_string();
                let timestamp = parts[3].to_string();

                let record = DataRecord::new(id, value, category, timestamp);
                if record.is_valid() {
                    self.records.push(record);
                    count += 1;
                }
            }
        }

        Ok(count)
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    pub fn calculate_average(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }

        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        sum / self.records.len() as f64
    }

    pub fn get_statistics(&self) -> (f64, f64, f64) {
        if self.records.is_empty() {
            return (0.0, 0.0, 0.0);
        }

        let values: Vec<f64> = self.records.iter().map(|r| r.value).collect();
        let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let avg = self.calculate_average();

        (min, max, avg)
    }

    pub fn export_summary(&self) -> String {
        let (min, max, avg) = self.get_statistics();
        format!(
            "Total records: {}\nMin value: {:.2}\nMax value: {:.2}\nAverage: {:.2}",
            self.records.len(),
            min,
            max,
            avg
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_validation() {
        let valid_record = DataRecord::new(1, 10.5, "A".to_string(), "2024-01-01".to_string());
        assert!(valid_record.is_valid());

        let invalid_record = DataRecord::new(0, -5.0, "".to_string(), "2024-01-01".to_string());
        assert!(!invalid_record.is_valid());
    }

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        assert_eq!(processor.records.len(), 0);
        assert_eq!(processor.calculate_average(), 0.0);
    }
}
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub tags: Vec<String>,
}

#[derive(Debug)]
pub enum ProcessingError {
    InvalidId,
    InvalidValue,
    EmptyName,
    DuplicateTag,
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessingError::InvalidId => write!(f, "ID must be greater than zero"),
            ProcessingError::InvalidValue => write!(f, "Value must be between 0.0 and 1000.0"),
            ProcessingError::EmptyName => write!(f, "Name cannot be empty"),
            ProcessingError::DuplicateTag => write!(f, "Duplicate tags are not allowed"),
        }
    }
}

impl Error for ProcessingError {}

impl DataRecord {
    pub fn new(id: u32, name: String, value: f64, tags: Vec<String>) -> Result<Self, ProcessingError> {
        if id == 0 {
            return Err(ProcessingError::InvalidId);
        }
        
        if name.trim().is_empty() {
            return Err(ProcessingError::EmptyName);
        }
        
        if value < 0.0 || value > 1000.0 {
            return Err(ProcessingError::InvalidValue);
        }
        
        let mut seen_tags = HashMap::new();
        for tag in &tags {
            if seen_tags.contains_key(tag) {
                return Err(ProcessingError::DuplicateTag);
            }
            seen_tags.insert(tag.clone(), true);
        }
        
        Ok(DataRecord {
            id,
            name,
            value,
            tags,
        })
    }
    
    pub fn normalize_value(&mut self) {
        self.value = (self.value * 100.0).round() / 100.0;
    }
    
    pub fn add_tag(&mut self, tag: String) -> Result<(), ProcessingError> {
        if self.tags.contains(&tag) {
            return Err(ProcessingError::DuplicateTag);
        }
        self.tags.push(tag);
        Ok(())
    }
    
    pub fn calculate_score(&self) -> f64 {
        let base_score = self.value / 10.0;
        let tag_bonus = self.tags.len() as f64 * 0.5;
        base_score + tag_bonus
    }
}

pub fn process_records(records: &mut [DataRecord]) -> Vec<Result<f64, ProcessingError>> {
    records.iter_mut().map(|record| {
        record.normalize_value();
        if record.value > 900.0 {
            Err(ProcessingError::InvalidValue)
        } else {
            Ok(record.calculate_score())
        }
    }).collect()
}

pub fn filter_records_by_threshold(records: &[DataRecord], threshold: f64) -> Vec<&DataRecord> {
    records.iter()
        .filter(|record| record.value >= threshold)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_valid_record_creation() {
        let record = DataRecord::new(
            1,
            "Test Record".to_string(),
            500.0,
            vec!["tag1".to_string(), "tag2".to_string()]
        ).unwrap();
        
        assert_eq!(record.id, 1);
        assert_eq!(record.name, "Test Record");
        assert_eq!(record.value, 500.0);
        assert_eq!(record.tags.len(), 2);
    }
    
    #[test]
    fn test_invalid_id() {
        let result = DataRecord::new(
            0,
            "Test".to_string(),
            100.0,
            vec![]
        );
        
        assert!(matches!(result, Err(ProcessingError::InvalidId)));
    }
    
    #[test]
    fn test_duplicate_tags() {
        let result = DataRecord::new(
            1,
            "Test".to_string(),
            100.0,
            vec!["tag1".to_string(), "tag1".to_string()]
        );
        
        assert!(matches!(result, Err(ProcessingError::DuplicateTag)));
    }
    
    #[test]
    fn test_calculate_score() {
        let record = DataRecord::new(
            1,
            "Test".to_string(),
            500.0,
            vec!["tag1".to_string(), "tag2".to_string(), "tag3".to_string()]
        ).unwrap();
        
        let score = record.calculate_score();
        assert_eq!(score, 50.0 + 1.5);
    }
}