
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug)]
pub enum DataError {
    InvalidId,
    InvalidName,
    InvalidValue,
    MissingMetadata,
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataError::InvalidId => write!(f, "ID must be greater than 0"),
            DataError::InvalidName => write!(f, "Name cannot be empty"),
            DataError::InvalidValue => write!(f, "Value must be between 0.0 and 1000.0"),
            DataError::MissingMetadata => write!(f, "Required metadata field is missing"),
        }
    }
}

impl Error for DataError {}

impl DataRecord {
    pub fn new(id: u32, name: String, value: f64) -> Result<Self, DataError> {
        if id == 0 {
            return Err(DataError::InvalidId);
        }
        if name.trim().is_empty() {
            return Err(DataError::InvalidName);
        }
        if !(0.0..=1000.0).contains(&value) {
            return Err(DataError::InvalidValue);
        }

        Ok(Self {
            id,
            name,
            value,
            metadata: HashMap::new(),
        })
    }

    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }

    pub fn validate_required_metadata(&self, required_keys: &[&str]) -> Result<(), DataError> {
        for key in required_keys {
            if !self.metadata.contains_key(*key) {
                return Err(DataError::MissingMetadata);
            }
        }
        Ok(())
    }

    pub fn transform_value(&mut self, multiplier: f64) -> Result<(), DataError> {
        let new_value = self.value * multiplier;
        if !(0.0..=1000.0).contains(&new_value) {
            return Err(DataError::InvalidValue);
        }
        self.value = new_value;
        Ok(())
    }

    pub fn calculate_score(&self) -> f64 {
        let base_score = self.value / 10.0;
        let metadata_bonus = self.metadata.len() as f64 * 0.5;
        base_score + metadata_bonus
    }
}

pub fn process_records(records: &mut [DataRecord]) -> Vec<Result<f64, DataError>> {
    let required_metadata = ["category", "source"];
    
    records
        .iter_mut()
        .map(|record| {
            record.validate_required_metadata(&required_metadata)?;
            record.transform_value(1.1)?;
            Ok(record.calculate_score())
        })
        .collect()
}

pub fn filter_records_by_threshold(records: &[DataRecord], threshold: f64) -> Vec<&DataRecord> {
    records
        .iter()
        .filter(|record| record.value >= threshold)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_record_creation() {
        let record = DataRecord::new(1, "Test".to_string(), 100.0);
        assert!(record.is_ok());
    }

    #[test]
    fn test_invalid_id() {
        let record = DataRecord::new(0, "Test".to_string(), 100.0);
        assert!(matches!(record, Err(DataError::InvalidId)));
    }

    #[test]
    fn test_metadata_operations() {
        let mut record = DataRecord::new(1, "Test".to_string(), 100.0).unwrap();
        record.add_metadata("category".to_string(), "sample".to_string());
        
        assert_eq!(record.get_metadata("category"), Some(&"sample".to_string()));
        assert!(record.validate_required_metadata(&["category"]).is_ok());
    }

    #[test]
    fn test_value_transformation() {
        let mut record = DataRecord::new(1, "Test".to_string(), 100.0).unwrap();
        assert!(record.transform_value(2.0).is_ok());
        assert_eq!(record.value, 200.0);
    }
}