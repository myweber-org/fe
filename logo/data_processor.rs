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
pub enum ValidationError {
    InvalidId,
    EmptyName,
    NegativeValue,
    DuplicateTag,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::InvalidId => write!(f, "ID must be greater than zero"),
            ValidationError::EmptyName => write!(f, "Name cannot be empty"),
            ValidationError::NegativeValue => write!(f, "Value cannot be negative"),
            ValidationError::DuplicateTag => write!(f, "Tags contain duplicates"),
        }
    }
}

impl Error for ValidationError {}

impl DataRecord {
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
        
        let mut seen = HashMap::new();
        for tag in &self.tags {
            if seen.insert(tag, true).is_some() {
                return Err(ValidationError::DuplicateTag);
            }
        }
        
        Ok(())
    }
    
    pub fn transform(&mut self, multiplier: f64) -> &mut Self {
        self.value *= multiplier;
        self.name = self.name.to_uppercase();
        self.tags.sort();
        self.tags.dedup();
        self
    }
}

pub fn process_records(records: &mut [DataRecord], multiplier: f64) -> Result<Vec<DataRecord>, ValidationError> {
    let mut processed = Vec::with_capacity(records.len());
    
    for record in records {
        record.validate()?;
        let mut transformed = record.clone();
        transformed.transform(multiplier);
        processed.push(transformed);
    }
    
    Ok(processed)
}

pub fn calculate_statistics(records: &[DataRecord]) -> (f64, f64, f64) {
    if records.is_empty() {
        return (0.0, 0.0, 0.0);
    }
    
    let sum: f64 = records.iter().map(|r| r.value).sum();
    let count = records.len() as f64;
    let mean = sum / count;
    
    let variance: f64 = records.iter()
        .map(|r| (r.value - mean).powi(2))
        .sum::<f64>() / count;
    
    let std_dev = variance.sqrt();
    
    (mean, variance, std_dev)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_valid_record() {
        let record = DataRecord {
            id: 1,
            name: "Test".to_string(),
            value: 100.0,
            tags: vec!["tag1".to_string(), "tag2".to_string()],
        };
        
        assert!(record.validate().is_ok());
    }
    
    #[test]
    fn test_invalid_id() {
        let record = DataRecord {
            id: 0,
            name: "Test".to_string(),
            value: 100.0,
            tags: vec![],
        };
        
        assert!(matches!(record.validate(), Err(ValidationError::InvalidId)));
    }
    
    #[test]
    fn test_transform_record() {
        let mut record = DataRecord {
            id: 1,
            name: "test".to_string(),
            value: 10.0,
            tags: vec!["b".to_string(), "a".to_string(), "b".to_string()],
        };
        
        record.transform(2.0);
        
        assert_eq!(record.name, "TEST");
        assert_eq!(record.value, 20.0);
        assert_eq!(record.tags, vec!["a".to_string(), "b".to_string()]);
    }
    
    #[test]
    fn test_statistics() {
        let records = vec![
            DataRecord { id: 1, name: "A".to_string(), value: 10.0, tags: vec![] },
            DataRecord { id: 2, name: "B".to_string(), value: 20.0, tags: vec![] },
            DataRecord { id: 3, name: "C".to_string(), value: 30.0, tags: vec![] },
        ];
        
        let (mean, variance, std_dev) = calculate_statistics(&records);
        
        assert_eq!(mean, 20.0);
        assert_eq!(variance, 66.66666666666667);
        assert_eq!(std_dev, 8.16496580927726);
    }
}