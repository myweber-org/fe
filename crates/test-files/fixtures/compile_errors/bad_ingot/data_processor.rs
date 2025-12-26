
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
pub enum DataError {
    InvalidId,
    InvalidName,
    InvalidValue,
    EmptyTags,
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataError::InvalidId => write!(f, "ID must be greater than zero"),
            DataError::InvalidName => write!(f, "Name cannot be empty"),
            DataError::InvalidValue => write!(f, "Value must be positive"),
            DataError::EmptyTags => write!(f, "Record must have at least one tag"),
        }
    }
}

impl Error for DataError {}

impl DataRecord {
    pub fn new(id: u32, name: String, value: f64, tags: Vec<String>) -> Result<Self, DataError> {
        if id == 0 {
            return Err(DataError::InvalidId);
        }
        if name.trim().is_empty() {
            return Err(DataError::InvalidName);
        }
        if value <= 0.0 {
            return Err(DataError::InvalidValue);
        }
        if tags.is_empty() {
            return Err(DataError::EmptyTags);
        }

        Ok(Self {
            id,
            name,
            value,
            tags,
        })
    }

    pub fn transform(&self, multiplier: f64) -> Self {
        Self {
            id: self.id,
            name: self.name.clone(),
            value: self.value * multiplier,
            tags: self.tags.clone(),
        }
    }

    pub fn summary(&self) -> String {
        format!(
            "Record {}: {} (value: {:.2}) with {} tags",
            self.id,
            self.name,
            self.value,
            self.tags.len()
        )
    }
}

pub struct DataProcessor {
    records: HashMap<u32, DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        Self {
            records: HashMap::new(),
        }
    }

    pub fn add_record(&mut self, record: DataRecord) -> Result<(), DataError> {
        if self.records.contains_key(&record.id) {
            return Err(DataError::InvalidId);
        }
        self.records.insert(record.id, record);
        Ok(())
    }

    pub fn process_records(&mut self, multiplier: f64) -> Vec<DataRecord> {
        let mut processed = Vec::new();
        for record in self.records.values() {
            processed.push(record.transform(multiplier));
        }
        processed
    }

    pub fn get_statistics(&self) -> (usize, f64, f64) {
        let count = self.records.len();
        let total_value: f64 = self.records.values().map(|r| r.value).sum();
        let avg_value = if count > 0 {
            total_value / count as f64
        } else {
            0.0
        };
        (count, total_value, avg_value)
    }

    pub fn find_by_tag(&self, tag: &str) -> Vec<&DataRecord> {
        self.records
            .values()
            .filter(|record| record.tags.iter().any(|t| t == tag))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_record_creation() {
        let record = DataRecord::new(
            1,
            "Test Record".to_string(),
            100.0,
            vec!["tag1".to_string(), "tag2".to_string()],
        );
        assert!(record.is_ok());
    }

    #[test]
    fn test_invalid_id() {
        let record = DataRecord::new(
            0,
            "Test".to_string(),
            50.0,
            vec!["tag".to_string()],
        );
        assert!(matches!(record, Err(DataError::InvalidId)));
    }

    #[test]
    fn test_data_processor_operations() {
        let mut processor = DataProcessor::new();
        let record = DataRecord::new(
            1,
            "Sample".to_string(),
            10.0,
            vec!["important".to_string()],
        ).unwrap();

        assert!(processor.add_record(record).is_ok());
        assert_eq!(processor.get_statistics().0, 1);
        
        let tagged = processor.find_by_tag("important");
        assert_eq!(tagged.len(), 1);
    }
}