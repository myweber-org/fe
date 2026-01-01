
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
            DataError::NegativeValue => write!(f, "Value cannot be negative"),
            DataError::DuplicateTag => write!(f, "Duplicate tags are not allowed"),
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
        
        let mut seen_tags = HashMap::new();
        for tag in &tags {
            if seen_tags.contains_key(tag) {
                return Err(DataError::DuplicateTag);
            }
            seen_tags.insert(tag.clone(), true);
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
    
    pub fn add_tag(&mut self, tag: String) -> Result<(), DataError> {
        if self.tags.contains(&tag) {
            return Err(DataError::DuplicateTag);
        }
        self.tags.push(tag);
        Ok(())
    }
    
    pub fn calculate_score(&self) -> f64 {
        let tag_bonus = self.tags.len() as f64 * 0.5;
        self.value + tag_bonus
    }
}

pub fn process_records(records: Vec<DataRecord>) -> Vec<DataRecord> {
    records
        .into_iter()
        .filter(|r| r.value > 10.0)
        .map(|r| r.transform(1.1))
        .collect()
}

pub fn find_best_record(records: &[DataRecord]) -> Option<&DataRecord> {
    records.iter().max_by(|a, b| {
        a.calculate_score()
            .partial_cmp(&b.calculate_score())
            .unwrap_or(std::cmp::Ordering::Equal)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_valid_record_creation() {
        let record = DataRecord::new(
            1,
            "Test Record".to_string(),
            15.5,
            vec!["tag1".to_string(), "tag2".to_string()],
        ).unwrap();
        
        assert_eq!(record.id, 1);
        assert_eq!(record.name, "Test Record");
        assert_eq!(record.value, 15.5);
        assert_eq!(record.tags.len(), 2);
    }
    
    #[test]
    fn test_invalid_id() {
        let result = DataRecord::new(
            0,
            "Test".to_string(),
            10.0,
            vec![],
        );
        assert!(matches!(result, Err(DataError::InvalidId)));
    }
    
    #[test]
    fn test_transform() {
        let record = DataRecord::new(1, "Test".to_string(), 10.0, vec![]).unwrap();
        let transformed = record.transform(2.0);
        assert_eq!(transformed.value, 20.0);
    }
    
    #[test]
    fn test_calculate_score() {
        let record = DataRecord::new(
            1,
            "Test".to_string(),
            10.0,
            vec!["a".to_string(), "b".to_string(), "c".to_string()],
        ).unwrap();
        
        let score = record.calculate_score();
        assert_eq!(score, 11.5); // 10.0 + (3 * 0.5)
    }
}