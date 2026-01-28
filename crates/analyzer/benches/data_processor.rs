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
    InvalidValue,
    EmptyName,
    DuplicateTag,
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataError::InvalidId => write!(f, "ID must be greater than 0"),
            DataError::InvalidValue => write!(f, "Value must be between 0.0 and 1000.0"),
            DataError::EmptyName => write!(f, "Name cannot be empty"),
            DataError::DuplicateTag => write!(f, "Duplicate tags are not allowed"),
        }
    }
}

impl Error for DataError {}

impl DataRecord {
    pub fn validate(&self) -> Result<(), DataError> {
        if self.id == 0 {
            return Err(DataError::InvalidId);
        }
        
        if self.name.trim().is_empty() {
            return Err(DataError::EmptyName);
        }
        
        if self.value < 0.0 || self.value > 1000.0 {
            return Err(DataError::InvalidValue);
        }
        
        let mut seen_tags = HashMap::new();
        for tag in &self.tags {
            if seen_tags.contains_key(tag) {
                return Err(DataError::DuplicateTag);
            }
            seen_tags.insert(tag.clone(), true);
        }
        
        Ok(())
    }
    
    pub fn transform(&mut self, multiplier: f64) -> Result<(), DataError> {
        self.validate()?;
        
        let new_value = self.value * multiplier;
        if new_value < 0.0 || new_value > 1000.0 {
            return Err(DataError::InvalidValue);
        }
        
        self.value = new_value;
        Ok(())
    }
    
    pub fn add_tag(&mut self, tag: String) -> Result<(), DataError> {
        if self.tags.contains(&tag) {
            return Err(DataError::DuplicateTag);
        }
        
        self.tags.push(tag);
        self.validate()
    }
}

pub fn process_records(records: &mut [DataRecord], multiplier: f64) -> Vec<Result<(), DataError>> {
    records
        .iter_mut()
        .map(|record| record.transform(multiplier))
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
    fn test_valid_record() {
        let record = DataRecord {
            id: 1,
            name: "Test".to_string(),
            value: 50.0,
            tags: vec!["tag1".to_string(), "tag2".to_string()],
        };
        
        assert!(record.validate().is_ok());
    }
    
    #[test]
    fn test_invalid_id() {
        let record = DataRecord {
            id: 0,
            name: "Test".to_string(),
            value: 50.0,
            tags: vec![],
        };
        
        assert!(matches!(record.validate(), Err(DataError::InvalidId)));
    }
    
    #[test]
    fn test_transform_valid() {
        let mut record = DataRecord {
            id: 1,
            name: "Test".to_string(),
            value: 50.0,
            tags: vec![],
        };
        
        assert!(record.transform(2.0).is_ok());
        assert_eq!(record.value, 100.0);
    }
}