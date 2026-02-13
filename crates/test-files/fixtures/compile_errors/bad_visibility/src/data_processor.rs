
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub category: String,
}

#[derive(Debug)]
pub enum DataError {
    InvalidValue(f64),
    InvalidCategory(String),
    EmptyDataset,
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataError::InvalidValue(v) => write!(f, "Invalid value: {}", v),
            DataError::InvalidCategory(c) => write!(f, "Invalid category: {}", c),
            DataError::EmptyDataset => write!(f, "Dataset is empty"),
        }
    }
}

impl Error for DataError {}

pub struct DataProcessor {
    records: Vec<DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    pub fn add_record(&mut self, record: DataRecord) -> Result<(), DataError> {
        Self::validate_record(&record)?;
        self.records.push(record);
        Ok(())
    }

    pub fn process_data(&self) -> Result<Vec<DataRecord>, DataError> {
        if self.records.is_empty() {
            return Err(DataError::EmptyDataset);
        }

        let mut processed = self.records.clone();
        processed.sort_by(|a, b| a.value.partial_cmp(&b.value).unwrap());

        Ok(processed)
    }

    pub fn calculate_statistics(&self) -> Result<(f64, f64, f64), DataError> {
        if self.records.is_empty() {
            return Err(DataError::EmptyDataset);
        }

        let values: Vec<f64> = self.records.iter().map(|r| r.value).collect();
        let sum: f64 = values.iter().sum();
        let count = values.len() as f64;
        let mean = sum / count;

        let variance: f64 = values.iter()
            .map(|&v| (v - mean).powi(2))
            .sum::<f64>() / count;

        let std_dev = variance.sqrt();

        Ok((mean, variance, std_dev))
    }

    fn validate_record(record: &DataRecord) -> Result<(), DataError> {
        if record.value.is_nan() || record.value.is_infinite() {
            return Err(DataError::InvalidValue(record.value));
        }

        if record.category.trim().is_empty() {
            return Err(DataError::InvalidCategory(record.category.clone()));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_record() {
        let record = DataRecord {
            id: 1,
            value: 42.5,
            category: "valid".to_string(),
        };

        assert!(DataProcessor::validate_record(&record).is_ok());
    }

    #[test]
    fn test_invalid_value() {
        let record = DataRecord {
            id: 1,
            value: f64::NAN,
            category: "test".to_string(),
        };

        assert!(DataProcessor::validate_record(&record).is_err());
    }

    #[test]
    fn test_empty_dataset() {
        let processor = DataProcessor::new();
        assert!(processor.process_data().is_err());
    }

    #[test]
    fn test_statistics_calculation() {
        let mut processor = DataProcessor::new();
        processor.add_record(DataRecord {
            id: 1,
            value: 10.0,
            category: "A".to_string(),
        }).unwrap();
        processor.add_record(DataRecord {
            id: 2,
            value: 20.0,
            category: "B".to_string(),
        }).unwrap();

        let stats = processor.calculate_statistics().unwrap();
        assert_eq!(stats.0, 15.0);
    }
}