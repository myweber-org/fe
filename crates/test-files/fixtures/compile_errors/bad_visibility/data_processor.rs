
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug)]
pub enum DataError {
    InvalidId,
    EmptyValues,
    ValueOutOfRange(f64),
    MissingMetadata(String),
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataError::InvalidId => write!(f, "Invalid record ID"),
            DataError::EmptyValues => write!(f, "Record values cannot be empty"),
            DataError::ValueOutOfRange(val) => write!(f, "Value {} is out of valid range", val),
            DataError::MissingMetadata(key) => write!(f, "Missing metadata key: {}", key),
        }
    }
}

impl Error for DataError {}

pub struct DataProcessor {
    records: Vec<DataRecord>,
    validation_enabled: bool,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
            validation_enabled: true,
        }
    }

    pub fn add_record(&mut self, record: DataRecord) -> Result<(), DataError> {
        if self.validation_enabled {
            self.validate_record(&record)?;
        }
        self.records.push(record);
        Ok(())
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), DataError> {
        if record.id == 0 {
            return Err(DataError::InvalidId);
        }

        if record.values.is_empty() {
            return Err(DataError::EmptyValues);
        }

        for &value in &record.values {
            if !value.is_finite() || value < 0.0 || value > 1000.0 {
                return Err(DataError::ValueOutOfRange(value));
            }
        }

        if !record.metadata.contains_key("source") {
            return Err(DataError::MissingMetadata("source".to_string()));
        }

        Ok(())
    }

    pub fn calculate_statistics(&self) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        
        if self.records.is_empty() {
            return stats;
        }

        let total_values: Vec<f64> = self.records
            .iter()
            .flat_map(|r| r.values.clone())
            .collect();

        if !total_values.is_empty() {
            let sum: f64 = total_values.iter().sum();
            let count = total_values.len() as f64;
            let mean = sum / count;
            
            let variance: f64 = total_values
                .iter()
                .map(|&v| (v - mean).powi(2))
                .sum::<f64>() / count;
            
            stats.insert("mean".to_string(), mean);
            stats.insert("variance".to_string(), variance);
            stats.insert("total_records".to_string(), self.records.len() as f64);
            stats.insert("total_values".to_string(), count);
        }

        stats
    }

    pub fn transform_values<F>(&mut self, transform_fn: F) 
    where
        F: Fn(f64) -> f64,
    {
        for record in &mut self.records {
            record.values = record.values
                .iter()
                .map(|&v| transform_fn(v))
                .collect();
        }
    }

    pub fn get_record_count(&self) -> usize {
        self.records.len()
    }

    pub fn enable_validation(&mut self, enabled: bool) {
        self.validation_enabled = enabled;
    }

    pub fn clear(&mut self) {
        self.records.clear();
    }
}

impl Default for DataProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_record() {
        let mut processor = DataProcessor::new();
        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), "test".to_string());
        
        let record = DataRecord {
            id: 1,
            values: vec![10.5, 20.3, 30.7],
            metadata,
        };

        assert!(processor.add_record(record).is_ok());
        assert_eq!(processor.get_record_count(), 1);
    }

    #[test]
    fn test_invalid_id() {
        let processor = DataProcessor::new();
        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), "test".to_string());
        
        let record = DataRecord {
            id: 0,
            values: vec![10.5],
            metadata,
        };

        assert!(processor.validate_record(&record).is_err());
    }

    #[test]
    fn test_statistics_calculation() {
        let mut processor = DataProcessor::new();
        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), "test".to_string());
        
        let records = vec![
            DataRecord {
                id: 1,
                values: vec![10.0, 20.0],
                metadata: metadata.clone(),
            },
            DataRecord {
                id: 2,
                values: vec![30.0, 40.0],
                metadata,
            },
        ];

        for record in records {
            processor.add_record(record).unwrap();
        }

        let stats = processor.calculate_statistics();
        assert_eq!(stats.get("mean").unwrap(), &25.0);
        assert_eq!(stats.get("total_records").unwrap(), &2.0);
    }
}