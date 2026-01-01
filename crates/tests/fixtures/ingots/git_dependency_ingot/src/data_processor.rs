use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u64,
    pub timestamp: i64,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

impl DataRecord {
    pub fn new(id: u64, timestamp: i64) -> Self {
        Self {
            id,
            timestamp,
            values: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn add_value(&mut self, value: f64) -> &mut Self {
        self.values.push(value);
        self
    }

    pub fn add_metadata(&mut self, key: &str, value: &str) -> &mut Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }

    pub fn validate(&self) -> Result<(), &'static str> {
        if self.id == 0 {
            return Err("Invalid record ID");
        }
        if self.timestamp < 0 {
            return Err("Timestamp cannot be negative");
        }
        if self.values.is_empty() {
            return Err("Record must contain at least one value");
        }
        Ok(())
    }

    pub fn calculate_statistics(&self) -> Option<DataStatistics> {
        if self.values.is_empty() {
            return None;
        }

        let count = self.values.len();
        let sum: f64 = self.values.iter().sum();
        let mean = sum / count as f64;
        let variance: f64 = self.values.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / count as f64;
        let std_dev = variance.sqrt();

        Some(DataStatistics {
            count,
            sum,
            mean,
            variance,
            std_dev,
        })
    }
}

#[derive(Debug, Clone)]
pub struct DataStatistics {
    pub count: usize,
    pub sum: f64,
    pub mean: f64,
    pub variance: f64,
    pub std_dev: f64,
}

pub fn process_records(records: &[DataRecord]) -> Vec<Result<DataStatistics, &'static str>> {
    records.iter()
        .map(|record| {
            record.validate()
                .and_then(|_| record.calculate_statistics()
                    .ok_or("Failed to calculate statistics"))
        })
        .collect()
}

pub fn filter_valid_records(records: &[DataRecord]) -> Vec<DataRecord> {
    records.iter()
        .filter(|record| record.validate().is_ok())
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_creation() {
        let mut record = DataRecord::new(1, 1234567890);
        record.add_value(10.5)
            .add_value(20.3)
            .add_metadata("source", "sensor_a");

        assert_eq!(record.id, 1);
        assert_eq!(record.values.len(), 2);
        assert_eq!(record.metadata.get("source"), Some(&"sensor_a".to_string()));
    }

    #[test]
    fn test_record_validation() {
        let valid_record = DataRecord::new(1, 1234567890);
        assert!(valid_record.add_value(10.0).validate().is_ok());

        let invalid_record = DataRecord::new(0, 1234567890);
        assert!(invalid_record.add_value(10.0).validate().is_err());
    }

    #[test]
    fn test_statistics_calculation() {
        let mut record = DataRecord::new(1, 1234567890);
        record.add_value(1.0).add_value(2.0).add_value(3.0);

        let stats = record.calculate_statistics().unwrap();
        assert_eq!(stats.count, 3);
        assert_eq!(stats.mean, 2.0);
        assert_eq!(stats.sum, 6.0);
    }
}use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub timestamp: i64,
}

#[derive(Debug)]
pub enum ValidationError {
    InvalidId,
    InvalidValue,
    InvalidTimestamp,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::InvalidId => write!(f, "ID must be greater than 0"),
            ValidationError::InvalidValue => write!(f, "Value must be between 0.0 and 1000.0"),
            ValidationError::InvalidTimestamp => write!(f, "Timestamp must be non-negative"),
        }
    }
}

impl Error for ValidationError {}

impl DataRecord {
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.id == 0 {
            return Err(ValidationError::InvalidId);
        }
        
        if self.value < 0.0 || self.value > 1000.0 {
            return Err(ValidationError::InvalidValue);
        }
        
        if self.timestamp < 0 {
            return Err(ValidationError::InvalidTimestamp);
        }
        
        Ok(())
    }
    
    pub fn transform(&mut self, multiplier: f64) -> Result<(), Box<dyn Error>> {
        self.validate()?;
        
        if multiplier <= 0.0 {
            return Err("Multiplier must be positive".into());
        }
        
        self.value *= multiplier;
        Ok(())
    }
}

pub fn process_records(records: &mut [DataRecord], multiplier: f64) -> Result<Vec<DataRecord>, Box<dyn Error>> {
    let mut processed = Vec::with_capacity(records.len());
    
    for record in records {
        let mut cloned = record.clone();
        cloned.transform(multiplier)?;
        processed.push(cloned);
    }
    
    Ok(processed)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_valid_record() {
        let record = DataRecord {
            id: 1,
            value: 100.0,
            timestamp: 1625097600,
        };
        
        assert!(record.validate().is_ok());
    }
    
    #[test]
    fn test_invalid_id() {
        let record = DataRecord {
            id: 0,
            value: 100.0,
            timestamp: 1625097600,
        };
        
        assert!(matches!(record.validate(), Err(ValidationError::InvalidId)));
    }
    
    #[test]
    fn test_transform_record() {
        let mut record = DataRecord {
            id: 1,
            value: 100.0,
            timestamp: 1625097600,
        };
        
        assert!(record.transform(2.0).is_ok());
        assert_eq!(record.value, 200.0);
    }
}