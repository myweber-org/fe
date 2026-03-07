
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct DataSet {
    values: Vec<f64>,
}

impl DataSet {
    pub fn new() -> Self {
        DataSet { values: Vec::new() }
    }

    pub fn from_csv<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut values = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if let Ok(value) = line.trim().parse::<f64>() {
                values.push(value);
            }
        }

        Ok(DataSet { values })
    }

    pub fn add_value(&mut self, value: f64) {
        self.values.push(value);
    }

    pub fn mean(&self) -> Option<f64> {
        if self.values.is_empty() {
            return None;
        }
        let sum: f64 = self.values.iter().sum();
        Some(sum / self.values.len() as f64)
    }

    pub fn variance(&self) -> Option<f64> {
        if self.values.len() < 2 {
            return None;
        }
        let mean = self.mean().unwrap();
        let sum_sq_diff: f64 = self.values.iter()
            .map(|&x| (x - mean).powi(2))
            .sum();
        Some(sum_sq_diff / (self.values.len() - 1) as f64)
    }

    pub fn standard_deviation(&self) -> Option<f64> {
        self.variance().map(|v| v.sqrt())
    }

    pub fn min(&self) -> Option<f64> {
        self.values.iter().copied().reduce(f64::min)
    }

    pub fn max(&self) -> Option<f64> {
        self.values.iter().copied().reduce(f64::max)
    }

    pub fn count(&self) -> usize {
        self.values.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_empty_dataset() {
        let ds = DataSet::new();
        assert_eq!(ds.mean(), None);
        assert_eq!(ds.count(), 0);
    }

    #[test]
    fn test_basic_statistics() {
        let mut ds = DataSet::new();
        ds.add_value(1.0);
        ds.add_value(2.0);
        ds.add_value(3.0);
        ds.add_value(4.0);
        ds.add_value(5.0);

        assert_eq!(ds.mean(), Some(3.0));
        assert_eq!(ds.variance(), Some(2.5));
        assert_eq!(ds.standard_deviation(), Some(2.5_f64.sqrt()));
        assert_eq!(ds.min(), Some(1.0));
        assert_eq!(ds.max(), Some(5.0));
        assert_eq!(ds.count(), 5);
    }

    #[test]
    fn test_csv_parsing() -> Result<(), Box<dyn Error>> {
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "10.5")?;
        writeln!(temp_file, "20.3")?;
        writeln!(temp_file, "15.7")?;
        writeln!(temp_file, "invalid")?;
        writeln!(temp_file, "25.1")?;

        let ds = DataSet::from_csv(temp_file.path())?;
        assert_eq!(ds.count(), 4);
        assert_eq!(ds.mean(), Some(17.9));
        Ok(())
    }
}
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProcessingError {
    #[error("Invalid input data")]
    InvalidData,
    #[error("Transformation failed: {0}")]
    TransformationFailed(String),
    #[error("Validation error: {0}")]
    ValidationError(String),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DataRecord {
    pub id: u64,
    pub timestamp: i64,
    pub values: HashMap<String, f64>,
    pub metadata: Option<HashMap<String, String>>,
}

impl DataRecord {
    pub fn new(id: u64, timestamp: i64) -> Self {
        Self {
            id,
            timestamp,
            values: HashMap::new(),
            metadata: None,
        }
    }

    pub fn add_value(&mut self, key: &str, value: f64) {
        self.values.insert(key.to_string(), value);
    }

    pub fn add_metadata(&mut self, key: &str, value: &str) {
        if self.metadata.is_none() {
            self.metadata = Some(HashMap::new());
        }
        if let Some(ref mut metadata) = self.metadata {
            metadata.insert(key.to_string(), value.to_string());
        }
    }

    pub fn validate(&self) -> Result<(), ProcessingError> {
        if self.id == 0 {
            return Err(ProcessingError::ValidationError(
                "ID cannot be zero".to_string(),
            ));
        }

        if self.timestamp < 0 {
            return Err(ProcessingError::ValidationError(
                "Timestamp cannot be negative".to_string(),
            ));
        }

        if self.values.is_empty() {
            return Err(ProcessingError::ValidationError(
                "Record must contain at least one value".to_string(),
            ));
        }

        for (key, value) in &self.values {
            if key.trim().is_empty() {
                return Err(ProcessingError::ValidationError(
                    "Value key cannot be empty".to_string(),
                ));
            }
            if !value.is_finite() {
                return Err(ProcessingError::ValidationError(format!(
                    "Value for key '{}' must be finite",
                    key
                )));
            }
        }

        Ok(())
    }

    pub fn normalize_values(&mut self) -> Result<(), ProcessingError> {
        if self.values.is_empty() {
            return Err(ProcessingError::TransformationFailed(
                "No values to normalize".to_string(),
            ));
        }

        let sum: f64 = self.values.values().sum();
        if sum == 0.0 {
            return Err(ProcessingError::TransformationFailed(
                "Cannot normalize zero sum".to_string(),
            ));
        }

        for value in self.values.values_mut() {
            *value /= sum;
        }

        Ok(())
    }

    pub fn calculate_statistics(&self) -> HashMap<String, f64> {
        let mut stats = HashMap::new();

        if self.values.is_empty() {
            return stats;
        }

        let values: Vec<f64> = self.values.values().copied().collect();
        let count = values.len() as f64;

        let sum: f64 = values.iter().sum();
        let mean = sum / count;

        let variance: f64 = values
            .iter()
            .map(|&v| (v - mean).powi(2))
            .sum::<f64>()
            / count;

        let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

        stats.insert("count".to_string(), count);
        stats.insert("sum".to_string(), sum);
        stats.insert("mean".to_string(), mean);
        stats.insert("variance".to_string(), variance);
        stats.insert("min".to_string(), min);
        stats.insert("max".to_string(), max);

        stats
    }
}

pub fn process_records(records: &mut [DataRecord]) -> Result<Vec<DataRecord>, ProcessingError> {
    let mut processed = Vec::with_capacity(records.len());

    for record in records.iter_mut() {
        record.validate()?;
        record.normalize_values()?;
        processed.push(record.clone());
    }

    Ok(processed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_creation() {
        let record = DataRecord::new(1, 1234567890);
        assert_eq!(record.id, 1);
        assert_eq!(record.timestamp, 1234567890);
        assert!(record.values.is_empty());
        assert!(record.metadata.is_none());
    }

    #[test]
    fn test_add_value() {
        let mut record = DataRecord::new(1, 1234567890);
        record.add_value("temperature", 25.5);
        assert_eq!(record.values.get("temperature"), Some(&25.5));
    }

    #[test]
    fn test_validation_success() {
        let mut record = DataRecord::new(1, 1234567890);
        record.add_value("temp", 25.5);
        assert!(record.validate().is_ok());
    }

    #[test]
    fn test_validation_failure() {
        let record = DataRecord::new(0, 1234567890);
        assert!(record.validate().is_err());
    }

    #[test]
    fn test_normalize_values() {
        let mut record = DataRecord::new(1, 1234567890);
        record.add_value("a", 1.0);
        record.add_value("b", 2.0);
        record.add_value("c", 3.0);

        assert!(record.normalize_values().is_ok());

        let expected_sum: f64 = record.values.values().sum();
        assert!((expected_sum - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_calculate_statistics() {
        let mut record = DataRecord::new(1, 1234567890);
        record.add_value("x", 1.0);
        record.add_value("y", 2.0);
        record.add_value("z", 3.0);

        let stats = record.calculate_statistics();
        assert_eq!(stats.get("count"), Some(&3.0));
        assert_eq!(stats.get("sum"), Some(&6.0));
        assert_eq!(stats.get("mean"), Some(&2.0));
    }
}