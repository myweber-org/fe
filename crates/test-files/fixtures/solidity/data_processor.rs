
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
            DataError::InvalidId => write!(f, "ID must be greater than 0"),
            DataError::InvalidName => write!(f, "Name cannot be empty"),
            DataError::InvalidValue => write!(f, "Value must be between 0.0 and 1000.0"),
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
        if value < 0.0 || value > 1000.0 {
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

    pub fn transform(&self) -> HashMap<String, String> {
        let mut result = HashMap::new();
        result.insert("identifier".to_string(), format!("REC-{:04}", self.id));
        result.insert("processed_name".to_string(), self.name.to_uppercase());
        result.insert("normalized_value".to_string(), format!("{:.2}", self.value / 100.0));
        result.insert("tag_count".to_string(), self.tags.len().to_string());
        result.insert("tag_summary".to_string(), self.tags.join("|"));
        result
    }

    pub fn validate_consistency(&self) -> bool {
        self.id > 0
            && !self.name.is_empty()
            && self.value >= 0.0
            && self.value <= 1000.0
            && !self.tags.is_empty()
    }
}

pub fn process_records(records: Vec<DataRecord>) -> Vec<HashMap<String, String>> {
    records
        .into_iter()
        .filter(|r| r.validate_consistency())
        .map(|r| r.transform())
        .collect()
}

pub fn calculate_statistics(records: &[DataRecord]) -> HashMap<String, f64> {
    let mut stats = HashMap::new();
    
    if records.is_empty() {
        return stats;
    }

    let count = records.len() as f64;
    let sum: f64 = records.iter().map(|r| r.value).sum();
    let avg = sum / count;
    
    let variance: f64 = records
        .iter()
        .map(|r| (r.value - avg).powi(2))
        .sum::<f64>() / count;
    
    stats.insert("record_count".to_string(), count);
    stats.insert("value_sum".to_string(), sum);
    stats.insert("value_average".to_string(), avg);
    stats.insert("value_variance".to_string(), variance);
    stats.insert("value_min".to_string(), records.iter().map(|r| r.value).fold(f64::INFINITY, f64::min));
    stats.insert("value_max".to_string(), records.iter().map(|r| r.value).fold(f64::NEG_INFINITY, f64::max));

    stats
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_record_creation() {
        let record = DataRecord::new(
            1,
            "Test Record".to_string(),
            250.5,
            vec!["tag1".to_string(), "tag2".to_string()],
        );
        assert!(record.is_ok());
    }

    #[test]
    fn test_invalid_id() {
        let record = DataRecord::new(
            0,
            "Test".to_string(),
            100.0,
            vec!["tag".to_string()],
        );
        assert!(matches!(record, Err(DataError::InvalidId)));
    }

    #[test]
    fn test_transform_output() {
        let record = DataRecord::new(
            42,
            "sample".to_string(),
            123.456,
            vec!["alpha".to_string(), "beta".to_string()],
        ).unwrap();
        
        let transformed = record.transform();
        assert_eq!(transformed.get("identifier").unwrap(), "REC-0042");
        assert_eq!(transformed.get("processed_name").unwrap(), "SAMPLE");
        assert_eq!(transformed.get("normalized_value").unwrap(), "1.23");
        assert_eq!(transformed.get("tag_count").unwrap(), "2");
    }

    #[test]
    fn test_statistics_calculation() {
        let records = vec![
            DataRecord::new(1, "A".to_string(), 100.0, vec!["t1".to_string()]).unwrap(),
            DataRecord::new(2, "B".to_string(), 200.0, vec!["t2".to_string()]).unwrap(),
            DataRecord::new(3, "C".to_string(), 300.0, vec!["t3".to_string()]).unwrap(),
        ];
        
        let stats = calculate_statistics(&records);
        assert_eq!(stats.get("record_count").unwrap(), &3.0);
        assert_eq!(stats.get("value_sum").unwrap(), &600.0);
        assert_eq!(stats.get("value_average").unwrap(), &200.0);
    }
}