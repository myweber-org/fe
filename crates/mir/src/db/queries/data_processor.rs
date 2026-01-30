
use std::collections::HashMap;

pub struct DataProcessor {
    cache: HashMap<String, Vec<f64>>,
    validation_rules: Vec<ValidationRule>,
}

pub struct ValidationRule {
    field_name: String,
    min_value: f64,
    max_value: f64,
    required: bool,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            cache: HashMap::new(),
            validation_rules: Vec::new(),
        }
    }

    pub fn add_validation_rule(&mut self, rule: ValidationRule) {
        self.validation_rules.push(rule);
    }

    pub fn process_dataset(&mut self, dataset_name: &str, data: Vec<f64>) -> Result<Vec<f64>, String> {
        if data.is_empty() {
            return Err("Dataset cannot be empty".to_string());
        }

        for rule in &self.validation_rules {
            if !self.validate_data(&data, rule) {
                return Err(format!("Validation failed for rule: {}", rule.field_name));
            }
        }

        let processed_data = self.transform_data(data);
        self.cache.insert(dataset_name.to_string(), processed_data.clone());
        
        Ok(processed_data)
    }

    fn validate_data(&self, data: &[f64], rule: &ValidationRule) -> bool {
        if rule.required && data.is_empty() {
            return false;
        }

        for &value in data {
            if value < rule.min_value || value > rule.max_value {
                return false;
            }
        }
        true
    }

    fn transform_data(&self, mut data: Vec<f64>) -> Vec<f64> {
        data.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        if let Some(mean) = self.calculate_mean(&data) {
            data.iter_mut().for_each(|x| *x = (*x - mean).abs());
        }
        
        data
    }

    fn calculate_mean(&self, data: &[f64]) -> Option<f64> {
        if data.is_empty() {
            return None;
        }
        
        let sum: f64 = data.iter().sum();
        Some(sum / data.len() as f64)
    }

    pub fn get_cached_data(&self, dataset_name: &str) -> Option<&Vec<f64>> {
        self.cache.get(dataset_name)
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
}

impl ValidationRule {
    pub fn new(field_name: &str, min_value: f64, max_value: f64, required: bool) -> Self {
        ValidationRule {
            field_name: field_name.to_string(),
            min_value,
            max_value,
            required,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_processing() {
        let mut processor = DataProcessor::new();
        let rule = ValidationRule::new("temperature", -50.0, 100.0, true);
        processor.add_validation_rule(rule);

        let data = vec![25.5, 30.2, 18.7, 22.1];
        let result = processor.process_dataset("weather_data", data);

        assert!(result.is_ok());
        assert!(processor.get_cached_data("weather_data").is_some());
    }

    #[test]
    fn test_validation_failure() {
        let mut processor = DataProcessor::new();
        let rule = ValidationRule::new("pressure", 0.0, 10.0, true);
        processor.add_validation_rule(rule);

        let invalid_data = vec![15.0, 5.0, 8.0];
        let result = processor.process_dataset("invalid_data", invalid_data);

        assert!(result.is_err());
    }
}
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    id: u32,
    name: String,
    value: f64,
    tags: Vec<String>,
}

#[derive(Debug)]
pub enum ProcessingError {
    InvalidData(String),
    TransformationError(String),
    ValidationError(String),
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessingError::InvalidData(msg) => write!(f, "Invalid data: {}", msg),
            ProcessingError::TransformationError(msg) => write!(f, "Transformation error: {}", msg),
            ProcessingError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl Error for ProcessingError {}

impl DataRecord {
    pub fn new(id: u32, name: String, value: f64, tags: Vec<String>) -> Result<Self, ProcessingError> {
        if name.trim().is_empty() {
            return Err(ProcessingError::InvalidData("Name cannot be empty".to_string()));
        }
        if value < 0.0 {
            return Err(ProcessingError::InvalidData("Value must be non-negative".to_string()));
        }
        
        Ok(Self {
            id,
            name,
            value,
            tags,
        })
    }

    pub fn transform(&mut self, multiplier: f64) -> Result<(), ProcessingError> {
        if multiplier <= 0.0 {
            return Err(ProcessingError::TransformationError(
                "Multiplier must be positive".to_string()
            ));
        }
        
        self.value *= multiplier;
        Ok(())
    }

    pub fn validate(&self) -> Result<(), ProcessingError> {
        if self.id == 0 {
            return Err(ProcessingError::ValidationError("ID cannot be zero".to_string()));
        }
        if self.name.len() > 100 {
            return Err(ProcessingError::ValidationError(
                "Name cannot exceed 100 characters".to_string()
            ));
        }
        Ok(())
    }

    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
    }

    pub fn get_normalized_value(&self, base: f64) -> f64 {
        if base == 0.0 {
            return 0.0;
        }
        self.value / base
    }
}

pub struct DataProcessor {
    records: HashMap<u32, DataRecord>,
    statistics: ProcessingStats,
}

#[derive(Debug, Default)]
pub struct ProcessingStats {
    pub total_records: usize,
    pub total_value: f64,
    pub average_value: f64,
}

impl DataProcessor {
    pub fn new() -> Self {
        Self {
            records: HashMap::new(),
            statistics: ProcessingStats::default(),
        }
    }

    pub fn add_record(&mut self, record: DataRecord) -> Result<(), ProcessingError> {
        record.validate()?;
        
        if self.records.contains_key(&record.id) {
            return Err(ProcessingError::ValidationError(
                format!("Record with ID {} already exists", record.id)
            ));
        }

        self.statistics.total_records += 1;
        self.statistics.total_value += record.value;
        self.statistics.average_value = self.statistics.total_value / self.statistics.total_records as f64;
        
        self.records.insert(record.id, record);
        Ok(())
    }

    pub fn process_batch(&mut self, multiplier: f64) -> Result<Vec<u32>, ProcessingError> {
        if multiplier <= 0.0 {
            return Err(ProcessingError::TransformationError(
                "Multiplier must be positive".to_string()
            ));
        }

        let mut processed_ids = Vec::new();
        for (id, record) in self.records.iter_mut() {
            record.transform(multiplier)?;
            processed_ids.push(*id);
        }

        self.update_statistics();
        Ok(processed_ids)
    }

    fn update_statistics(&mut self) {
        self.statistics.total_value = self.records.values().map(|r| r.value).sum();
        self.statistics.average_value = if self.statistics.total_records > 0 {
            self.statistics.total_value / self.statistics.total_records as f64
        } else {
            0.0
        };
    }

    pub fn get_record(&self, id: u32) -> Option<&DataRecord> {
        self.records.get(&id)
    }

    pub fn get_statistics(&self) -> &ProcessingStats {
        &self.statistics
    }

    pub fn filter_by_tag(&self, tag: &str) -> Vec<&DataRecord> {
        self.records
            .values()
            .filter(|record| record.tags.contains(&tag.to_string()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_creation() {
        let record = DataRecord::new(1, "Test".to_string(), 100.0, vec!["tag1".to_string()]);
        assert!(record.is_ok());
        
        let invalid_record = DataRecord::new(0, "".to_string(), -10.0, vec![]);
        assert!(invalid_record.is_err());
    }

    #[test]
    fn test_record_validation() {
        let record = DataRecord::new(1, "Valid".to_string(), 50.0, vec![]).unwrap();
        assert!(record.validate().is_ok());
    }

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        let record = DataRecord::new(1, "Record1".to_string(), 100.0, vec!["test".to_string()]).unwrap();
        
        assert!(processor.add_record(record).is_ok());
        assert_eq!(processor.get_statistics().total_records, 1);
    }

    #[test]
    fn test_batch_processing() {
        let mut processor = DataProcessor::new();
        let record1 = DataRecord::new(1, "R1".to_string(), 100.0, vec![]).unwrap();
        let record2 = DataRecord::new(2, "R2".to_string(), 200.0, vec![]).unwrap();
        
        processor.add_record(record1).unwrap();
        processor.add_record(record2).unwrap();
        
        let result = processor.process_batch(2.0);
        assert!(result.is_ok());
        
        let stats = processor.get_statistics();
        assert_eq!(stats.total_value, 600.0);
    }
}
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DataRecord {
    id: u64,
    timestamp: i64,
    values: Vec<f64>,
    metadata: HashMap<String, String>,
}

impl DataRecord {
    pub fn new(id: u64, timestamp: i64, values: Vec<f64>) -> Self {
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

    pub fn validate(&self) -> Result<(), String> {
        if self.id == 0 {
            return Err("Invalid record ID".to_string());
        }
        if self.timestamp < 0 {
            return Err("Timestamp cannot be negative".to_string());
        }
        if self.values.is_empty() {
            return Err("Values array cannot be empty".to_string());
        }
        Ok(())
    }

    pub fn transform_values<F>(&mut self, transformer: F)
    where
        F: Fn(f64) -> f64,
    {
        self.values = self.values.iter().map(|&v| transformer(v)).collect();
    }
}

pub fn process_records(records: &mut [DataRecord]) -> Vec<Result<DataRecord, String>> {
    records
        .iter_mut()
        .map(|record| {
            record.validate()?;
            record.transform_values(|v| v * 2.0);
            Ok(record.clone())
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_validation() {
        let valid_record = DataRecord::new(1, 1234567890, vec![1.0, 2.0, 3.0]);
        assert!(valid_record.validate().is_ok());

        let invalid_record = DataRecord::new(0, -1, vec![]);
        assert!(invalid_record.validate().is_err());
    }

    #[test]
    fn test_value_transformation() {
        let mut record = DataRecord::new(1, 1234567890, vec![1.0, 2.0, 3.0]);
        record.transform_values(|v| v * 3.0);
        assert_eq!(record.values, vec![3.0, 6.0, 9.0]);
    }
}