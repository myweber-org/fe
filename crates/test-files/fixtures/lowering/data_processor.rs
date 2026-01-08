
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
pub enum DataError {
    InvalidId,
    EmptyName,
    NegativeValue,
    DuplicateTag,
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataError::InvalidId => write!(f, "ID must be greater than zero"),
            DataError::EmptyName => write!(f, "Name cannot be empty"),
            DataError::NegativeValue => write!(f, "Value must be non-negative"),
            DataError::DuplicateTag => write!(f, "Tags must be unique"),
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
            return Err(DataError::EmptyName);
        }
        if value < 0.0 {
            return Err(DataError::NegativeValue);
        }
        
        let mut seen_tags = std::collections::HashSet::new();
        for tag in &tags {
            if !seen_tags.insert(tag) {
                return Err(DataError::DuplicateTag);
            }
        }
        
        Ok(Self {
            id,
            name,
            value,
            tags,
        })
    }
    
    pub fn transform_value(&mut self, multiplier: f64) {
        self.value *= multiplier;
    }
    
    pub fn add_tag(&mut self, tag: String) -> Result<(), DataError> {
        if self.tags.contains(&tag) {
            return Err(DataError::DuplicateTag);
        }
        self.tags.push(tag);
        Ok(())
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
    
    pub fn get_record(&self, id: u32) -> Option<&DataRecord> {
        self.records.get(&id)
    }
    
    pub fn calculate_total_value(&self) -> f64 {
        self.records.values().map(|r| r.value).sum()
    }
    
    pub fn find_by_tag(&self, tag: &str) -> Vec<&DataRecord> {
        self.records
            .values()
            .filter(|r| r.tags.contains(&tag.to_string()))
            .collect()
    }
    
    pub fn apply_to_all<F>(&mut self, mut transform: F)
    where
        F: FnMut(&mut DataRecord),
    {
        for record in self.records.values_mut() {
            transform(record);
        }
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
            42.5,
            vec!["tag1".to_string(), "tag2".to_string()],
        );
        assert!(record.is_ok());
    }
    
    #[test]
    fn test_invalid_id() {
        let record = DataRecord::new(
            0,
            "Test".to_string(),
            10.0,
            vec![],
        );
        assert!(matches!(record, Err(DataError::InvalidId)));
    }
    
    #[test]
    fn test_duplicate_tags() {
        let record = DataRecord::new(
            1,
            "Test".to_string(),
            10.0,
            vec!["tag1".to_string(), "tag1".to_string()],
        );
        assert!(matches!(record, Err(DataError::DuplicateTag)));
    }
    
    #[test]
    fn test_data_processor_operations() {
        let mut processor = DataProcessor::new();
        
        let record1 = DataRecord::new(
            1,
            "Record 1".to_string(),
            100.0,
            vec!["important".to_string()],
        ).unwrap();
        
        let record2 = DataRecord::new(
            2,
            "Record 2".to_string(),
            200.0,
            vec!["important".to_string(), "urgent".to_string()],
        ).unwrap();
        
        processor.add_record(record1).unwrap();
        processor.add_record(record2).unwrap();
        
        assert_eq!(processor.calculate_total_value(), 300.0);
        assert_eq!(processor.find_by_tag("important").len(), 2);
        assert_eq!(processor.find_by_tag("urgent").len(), 1);
    }
}