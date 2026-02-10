
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
pub enum DataError {
    #[error("Invalid timestamp: {0}")]
    InvalidTimestamp(i64),
    #[error("Empty values array")]
    EmptyValues,
    #[error("NaN value detected at index {0}")]
    NaNValue(usize),
    #[error("Duplicate record ID: {0}")]
    DuplicateId(u64),
}

pub struct DataProcessor {
    processed_ids: std::collections::HashSet<u64>,
    statistics: ProcessingStats,
}

#[derive(Debug, Default)]
pub struct ProcessingStats {
    pub total_records: usize,
    pub valid_records: usize,
    pub invalid_records: usize,
    pub average_values_per_record: f64,
}

impl DataProcessor {
    pub fn new() -> Self {
        Self {
            processed_ids: std::collections::HashSet::new(),
            statistics: ProcessingStats::default(),
        }
    }

    pub fn process_record(&mut self, record: &DataRecord) -> Result<DataRecord, DataError> {
        self.statistics.total_records += 1;

        if self.processed_ids.contains(&record.id) {
            self.statistics.invalid_records += 1;
            return Err(DataError::DuplicateId(record.id));
        }

        self.validate_timestamp(record.timestamp)?;
        self.validate_values(&record.values)?;

        let processed_record = self.transform_record(record);
        self.processed_ids.insert(record.id);
        self.statistics.valid_records += 1;

        self.update_statistics(&record);
        Ok(processed_record)
    }

    fn validate_timestamp(&self, timestamp: i64) -> Result<(), DataError> {
        if timestamp < 0 {
            Err(DataError::InvalidTimestamp(timestamp))
        } else {
            Ok(())
        }
    }

    fn validate_values(&self, values: &[f64]) -> Result<(), DataError> {
        if values.is_empty() {
            return Err(DataError::EmptyValues);
        }

        for (index, &value) in values.iter().enumerate() {
            if value.is_nan() {
                return Err(DataError::NaNValue(index));
            }
        }

        Ok(())
    }

    fn transform_record(&self, record: &DataRecord) -> DataRecord {
        let normalized_values: Vec<f64> = record
            .values
            .iter()
            .map(|&v| v.clamp(0.0, 1.0))
            .collect();

        let mut enhanced_metadata = record.metadata.clone();
        enhanced_metadata.insert(
            "processed_timestamp".to_string(),
            chrono::Utc::now().timestamp().to_string(),
        );
        enhanced_metadata.insert("values_count".to_string(), record.values.len().to_string());

        DataRecord {
            id: record.id,
            timestamp: record.timestamp,
            values: normalized_values,
            metadata: enhanced_metadata,
        }
    }

    fn update_statistics(&mut self, record: &DataRecord) {
        let total_values = self.statistics.valid_records * record.values.len();
        self.statistics.average_values_per_record = if self.statistics.valid_records > 0 {
            total_values as f64 / self.statistics.valid_records as f64
        } else {
            0.0
        };
    }

    pub fn get_statistics(&self) -> &ProcessingStats {
        &self.statistics
    }

    pub fn reset(&mut self) {
        self.processed_ids.clear();
        self.statistics = ProcessingStats::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_record_processing() {
        let mut processor = DataProcessor::new();
        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), "test".to_string());

        let record = DataRecord {
            id: 1,
            timestamp: 1625097600,
            values: vec![0.5, 0.8, 0.2],
            metadata,
        };

        let result = processor.process_record(&record);
        assert!(result.is_ok());
        assert_eq!(processor.get_statistics().valid_records, 1);
    }

    #[test]
    fn test_invalid_timestamp() {
        let mut processor = DataProcessor::new();
        let record = DataRecord {
            id: 1,
            timestamp: -1,
            values: vec![0.5],
            metadata: HashMap::new(),
        };

        let result = processor.process_record(&record);
        assert!(matches!(result, Err(DataError::InvalidTimestamp(-1))));
    }

    #[test]
    fn test_duplicate_id() {
        let mut processor = DataProcessor::new();
        let record1 = DataRecord {
            id: 1,
            timestamp: 1625097600,
            values: vec![0.5],
            metadata: HashMap::new(),
        };

        let record2 = DataRecord {
            id: 1,
            timestamp: 1625097601,
            values: vec![0.6],
            metadata: HashMap::new(),
        };

        let _ = processor.process_record(&record1);
        let result = processor.process_record(&record2);
        assert!(matches!(result, Err(DataError::DuplicateId(1))));
    }
}
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Debug, Serialize, Deserialize)]
pub struct DataRecord {
    id: u32,
    value: f64,
    category: String,
}

pub fn validate_record(record: &DataRecord) -> Result<(), Box<dyn Error>> {
    if record.id == 0 {
        return Err("ID cannot be zero".into());
    }
    
    if record.value < 0.0 {
        return Err("Value must be non-negative".into());
    }
    
    if record.category.is_empty() {
        return Err("Category cannot be empty".into());
    }
    
    Ok(())
}

pub fn transform_value(record: &mut DataRecord, multiplier: f64) {
    record.value *= multiplier;
}

pub fn filter_records(records: Vec<DataRecord>, min_value: f64) -> Vec<DataRecord> {
    records.into_iter()
        .filter(|r| r.value >= min_value)
        .collect()
}

pub fn calculate_average(records: &[DataRecord]) -> Option<f64> {
    if records.is_empty() {
        return None;
    }
    
    let sum: f64 = records.iter().map(|r| r.value).sum();
    Some(sum / records.len() as f64)
}
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct ValidationError {
    message: String,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Validation error: {}", self.message)
    }
}

impl Error for ValidationError {}

pub struct DataProcessor {
    threshold: f64,
}

impl DataProcessor {
    pub fn new(threshold: f64) -> Result<Self, ValidationError> {
        if threshold < 0.0 || threshold > 1.0 {
            return Err(ValidationError {
                message: format!("Threshold {} must be between 0.0 and 1.0", threshold),
            });
        }
        Ok(DataProcessor { threshold })
    }

    pub fn process_data(&self, input: Vec<f64>) -> Result<Vec<f64>, ValidationError> {
        if input.is_empty() {
            return Err(ValidationError {
                message: "Input data cannot be empty".to_string(),
            });
        }

        let filtered: Vec<f64> = input
            .into_iter()
            .filter(|&value| value >= self.threshold)
            .collect();

        if filtered.is_empty() {
            return Err(ValidationError {
                message: format!("No values above threshold {}", self.threshold),
            });
        }

        let max_value = filtered.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let normalized: Vec<f64> = filtered
            .iter()
            .map(|&value| value / max_value)
            .collect();

        Ok(normalized)
    }

    pub fn calculate_statistics(&self, data: &[f64]) -> (f64, f64, f64) {
        let count = data.len() as f64;
        let sum: f64 = data.iter().sum();
        let mean = sum / count;

        let variance: f64 = data
            .iter()
            .map(|value| {
                let diff = mean - value;
                diff * diff
            })
            .sum::<f64>()
            / count;

        let std_dev = variance.sqrt();

        (mean, variance, std_dev)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_processor_creation() {
        let processor = DataProcessor::new(0.5);
        assert!(processor.is_ok());
    }

    #[test]
    fn test_invalid_processor_creation() {
        let processor = DataProcessor::new(1.5);
        assert!(processor.is_err());
    }

    #[test]
    fn test_data_processing() {
        let processor = DataProcessor::new(0.3).unwrap();
        let input = vec![0.1, 0.4, 0.5, 0.2, 0.8];
        let result = processor.process_data(input);
        assert!(result.is_ok());
        let processed = result.unwrap();
        assert_eq!(processed.len(), 3);
    }

    #[test]
    fn test_statistics_calculation() {
        let processor = DataProcessor::new(0.0).unwrap();
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let (mean, variance, std_dev) = processor.calculate_statistics(&data);
        assert_eq!(mean, 3.0);
        assert_eq!(variance, 2.0);
        assert_eq!(std_dev, 2.0_f64.sqrt());
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, PartialEq)]
pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub category: String,
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

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            
            if line_num == 0 {
                continue;
            }

            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() != 3 {
                continue;
            }

            let id = match parts[0].parse::<u32>() {
                Ok(id) => id,
                Err(_) => continue,
            };

            let value = match parts[1].parse::<f64>() {
                Ok(value) => value,
                Err(_) => continue,
            };

            let category = parts[2].trim().to_string();

            if !self.validate_record(&category, value) {
                continue;
            }

            self.records.push(DataRecord {
                id,
                value,
                category,
            });
            count += 1;
        }

        Ok(count)
    }

    fn validate_record(&self, category: &str, value: f64) -> bool {
        !category.is_empty() && value >= 0.0 && value <= 1000.0
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }

        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    pub fn get_statistics(&self) -> Statistics {
        if self.records.is_empty() {
            return Statistics::default();
        }

        let values: Vec<f64> = self.records.iter().map(|r| r.value).collect();
        let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let sum: f64 = values.iter().sum();
        let count = values.len();

        Statistics {
            count,
            min,
            max,
            sum,
        }
    }
}

#[derive(Debug, Default)]
pub struct Statistics {
    pub count: usize,
    pub min: f64,
    pub max: f64,
    pub sum: f64,
}

impl Statistics {
    pub fn average(&self) -> Option<f64> {
        if self.count == 0 {
            None
        } else {
            Some(self.sum / self.count as f64)
        }
    }
}
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DataError {
    #[error("Invalid data format")]
    InvalidFormat,
    #[error("Missing required field: {0}")]
    MissingField(String),
    #[error("Validation failed: {0}")]
    ValidationFailed(String),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DataRecord {
    pub id: u64,
    pub timestamp: i64,
    pub values: HashMap<String, f64>,
    pub tags: Vec<String>,
}

impl DataRecord {
    pub fn validate(&self) -> Result<(), DataError> {
        if self.id == 0 {
            return Err(DataError::ValidationFailed("ID cannot be zero".to_string()));
        }
        
        if self.timestamp < 0 {
            return Err(DataError::ValidationFailed("Timestamp cannot be negative".to_string()));
        }
        
        if self.values.is_empty() {
            return Err(DataError::ValidationFailed("Values cannot be empty".to_string()));
        }
        
        for (key, value) in &self.values {
            if key.trim().is_empty() {
                return Err(DataError::ValidationFailed("Key cannot be empty".to_string()));
            }
            if !value.is_finite() {
                return Err(DataError::ValidationFailed(
                    format!("Value for key '{}' must be finite", key)
                ));
            }
        }
        
        Ok(())
    }
    
    pub fn transform(&mut self, multiplier: f64) -> Result<(), DataError> {
        if !multiplier.is_finite() || multiplier == 0.0 {
            return Err(DataError::ValidationFailed(
                "Multiplier must be finite and non-zero".to_string()
            ));
        }
        
        for value in self.values.values_mut() {
            *value *= multiplier;
        }
        
        Ok(())
    }
    
    pub fn calculate_statistics(&self) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        
        if self.values.is_empty() {
            return stats;
        }
        
        let values: Vec<f64> = self.values.values().copied().collect();
        let count = values.len() as f64;
        let sum: f64 = values.iter().sum();
        let mean = sum / count;
        
        let variance: f64 = values.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / count;
        
        stats.insert("count".to_string(), count);
        stats.insert("sum".to_string(), sum);
        stats.insert("mean".to_string(), mean);
        stats.insert("variance".to_string(), variance);
        
        if let Some(&min) = values.iter().min_by(|a, b| a.partial_cmp(b).unwrap()) {
            stats.insert("min".to_string(), min);
        }
        
        if let Some(&max) = values.iter().max_by(|a, b| a.partial_cmp(b).unwrap()) {
            stats.insert("max".to_string(), max);
        }
        
        stats
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
    
    pub fn add_record(&mut self, record: DataRecord) -> Result<(), DataError> {
        record.validate()?;
        self.records.push(record);
        Ok(())
    }
    
    pub fn process_all(&mut self, multiplier: f64) -> Result<(), DataError> {
        for record in &mut self.records {
            record.transform(multiplier)?;
        }
        Ok(())
    }
    
    pub fn get_aggregated_stats(&self) -> HashMap<String, f64> {
        let mut aggregated = HashMap::new();
        let mut total_count = 0.0;
        let mut weighted_sum = 0.0;
        
        for record in &self.records {
            let stats = record.calculate_statistics();
            let count = stats.get("count").copied().unwrap_or(0.0);
            
            if let Some(mean) = stats.get("mean") {
                weighted_sum += mean * count;
                total_count += count;
            }
        }
        
        if total_count > 0.0 {
            aggregated.insert("overall_mean".to_string(), weighted_sum / total_count);
        }
        
        aggregated.insert("total_records".to_string(), self.records.len() as f64);
        aggregated
    }
    
    pub fn filter_by_tag(&self, tag: &str) -> Vec<DataRecord> {
        self.records.iter()
            .filter(|record| record.tags.contains(&tag.to_string()))
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_record_validation() {
        let mut values = HashMap::new();
        values.insert("temperature".to_string(), 25.5);
        
        let valid_record = DataRecord {
            id: 1,
            timestamp: 1625097600,
            values: values.clone(),
            tags: vec!["sensor".to_string()],
        };
        
        assert!(valid_record.validate().is_ok());
        
        let invalid_record = DataRecord {
            id: 0,
            timestamp: 1625097600,
            values: values,
            tags: vec![],
        };
        
        assert!(invalid_record.validate().is_err());
    }
    
    #[test]
    fn test_data_transformation() {
        let mut values = HashMap::new();
        values.insert("value".to_string(), 10.0);
        
        let mut record = DataRecord {
            id: 1,
            timestamp: 1625097600,
            values,
            tags: vec![],
        };
        
        assert!(record.transform(2.0).is_ok());
        assert_eq!(record.values.get("value"), Some(&20.0));
    }
    
    #[test]
    fn test_statistics_calculation() {
        let mut values = HashMap::new();
        values.insert("a".to_string(), 1.0);
        values.insert("b".to_string(), 2.0);
        values.insert("c".to_string(), 3.0);
        
        let record = DataRecord {
            id: 1,
            timestamp: 1625097600,
            values,
            tags: vec![],
        };
        
        let stats = record.calculate_statistics();
        assert_eq!(stats.get("count"), Some(&3.0));
        assert_eq!(stats.get("mean"), Some(&2.0));
        assert_eq!(stats.get("sum"), Some(&6.0));
    }
}