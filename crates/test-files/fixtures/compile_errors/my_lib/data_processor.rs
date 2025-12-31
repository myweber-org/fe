
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct ProcessingError {
    details: String,
}

impl ProcessingError {
    fn new(msg: &str) -> ProcessingError {
        ProcessingError {
            details: msg.to_string(),
        }
    }
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl Error for ProcessingError {
    fn description(&self) -> &str {
        &self.details
    }
}

pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub timestamp: i64,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, timestamp: i64) -> Result<DataRecord, ProcessingError> {
        if value < 0.0 || value > 1000.0 {
            return Err(ProcessingError::new("Value must be between 0 and 1000"));
        }
        if timestamp < 0 {
            return Err(ProcessingError::new("Timestamp cannot be negative"));
        }
        Ok(DataRecord {
            id,
            value,
            timestamp,
        })
    }

    pub fn normalize(&self) -> f64 {
        (self.value - 0.0) / (1000.0 - 0.0)
    }

    pub fn is_anomaly(&self, threshold: f64) -> bool {
        self.normalize() > threshold
    }
}

pub fn process_records(records: &[DataRecord]) -> Vec<f64> {
    records.iter().map(|r| r.normalize()).collect()
}

pub fn filter_anomalies(records: &[DataRecord], threshold: f64) -> Vec<&DataRecord> {
    records
        .iter()
        .filter(|r| r.is_anomaly(threshold))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_record_creation() {
        let record = DataRecord::new(1, 500.0, 1234567890).unwrap();
        assert_eq!(record.id, 1);
        assert_eq!(record.value, 500.0);
        assert_eq!(record.timestamp, 1234567890);
    }

    #[test]
    fn test_invalid_value() {
        let result = DataRecord::new(1, 1500.0, 1234567890);
        assert!(result.is_err());
    }

    #[test]
    fn test_normalize() {
        let record = DataRecord::new(1, 500.0, 1234567890).unwrap();
        assert_eq!(record.normalize(), 0.5);
    }

    #[test]
    fn test_anomaly_detection() {
        let record = DataRecord::new(1, 900.0, 1234567890).unwrap();
        assert!(record.is_anomaly(0.8));
        assert!(!record.is_anomaly(0.9));
    }
}