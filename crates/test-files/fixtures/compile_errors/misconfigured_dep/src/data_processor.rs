
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    id: u32,
    name: String,
    value: f64,
    tags: Vec<String>,
    metadata: HashMap<String, String>,
}

#[derive(Debug)]
pub enum ValidationError {
    InvalidId,
    EmptyName,
    NegativeValue,
    MissingRequiredTag,
    DuplicateMetadataKey,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::InvalidId => write!(f, "ID must be greater than zero"),
            ValidationError::EmptyName => write!(f, "Name cannot be empty"),
            ValidationError::NegativeValue => write!(f, "Value cannot be negative"),
            ValidationError::MissingRequiredTag => write!(f, "Required tag is missing"),
            ValidationError::DuplicateMetadataKey => write!(f, "Duplicate metadata key found"),
        }
    }
}

impl Error for ValidationError {}

impl DataRecord {
    pub fn new(id: u32, name: String, value: f64) -> Self {
        DataRecord {
            id,
            name,
            value,
            tags: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.id == 0 {
            return Err(ValidationError::InvalidId);
        }
        
        if self.name.trim().is_empty() {
            return Err(ValidationError::EmptyName);
        }
        
        if self.value < 0.0 {
            return Err(ValidationError::NegativeValue);
        }
        
        if !self.tags.iter().any(|tag| tag == "processed") {
            return Err(ValidationError::MissingRequiredTag);
        }
        
        Ok(())
    }

    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
    }

    pub fn add_metadata(&mut self, key: String, value: String) -> Result<(), ValidationError> {
        if self.metadata.contains_key(&key) {
            return Err(ValidationError::DuplicateMetadataKey);
        }
        self.metadata.insert(key, value);
        Ok(())
    }

    pub fn transform_value<F>(&mut self, transformer: F)
    where
        F: Fn(f64) -> f64,
    {
        self.value = transformer(self.value);
    }

    pub fn calculate_score(&self) -> f64 {
        let base_score = self.value * 100.0;
        let tag_bonus = self.tags.len() as f64 * 10.0;
        let metadata_bonus = self.metadata.len() as f64 * 5.0;
        
        base_score + tag_bonus + metadata_bonus
    }

    pub fn to_json(&self) -> String {
        let tags_json = serde_json::to_string(&self.tags).unwrap_or_default();
        let metadata_json = serde_json::to_string(&self.metadata).unwrap_or_default();
        
        format!(
            r#"{{"id":{},"name":"{}","value":{},"tags":{},"metadata":{}}}"#,
            self.id, self.name, self.value, tags_json, metadata_json
        )
    }
}

pub fn process_records(records: &mut [DataRecord]) -> Vec<Result<f64, ValidationError>> {
    records
        .iter_mut()
        .map(|record| {
            record.add_tag("processed".to_string());
            record.transform_value(|v| v * 1.1);
            record.validate().map(|_| record.calculate_score())
        })
        .collect()
}

pub fn filter_valid_records(records: &[DataRecord]) -> Vec<&DataRecord> {
    records
        .iter()
        .filter(|record| record.validate().is_ok())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_creation() {
        let record = DataRecord::new(1, "Test Record".to_string(), 42.5);
        assert_eq!(record.id, 1);
        assert_eq!(record.name, "Test Record");
        assert_eq!(record.value, 42.5);
    }

    #[test]
    fn test_validation() {
        let mut record = DataRecord::new(1, "Valid".to_string(), 10.0);
        record.add_tag("processed".to_string());
        assert!(record.validate().is_ok());

        let invalid_record = DataRecord::new(0, "".to_string(), -5.0);
        assert!(invalid_record.validate().is_err());
    }

    #[test]
    fn test_value_transformation() {
        let mut record = DataRecord::new(1, "Test".to_string(), 10.0);
        record.transform_value(|v| v * 2.0);
        assert_eq!(record.value, 20.0);
    }

    #[test]
    fn test_score_calculation() {
        let mut record = DataRecord::new(1, "Test".to_string(), 10.0);
        record.add_tag("tag1".to_string());
        record.add_tag("tag2".to_string());
        record.add_metadata("key".to_string(), "value".to_string()).unwrap();
        
        let score = record.calculate_score();
        assert_eq!(score, 10.0 * 100.0 + 2.0 * 10.0 + 1.0 * 5.0);
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

pub fn filter_by_category(records: Vec<Record>, category: &str) -> Vec<Record> {
    records.into_iter()
        .filter(|r| r.category == category)
        .collect()
}