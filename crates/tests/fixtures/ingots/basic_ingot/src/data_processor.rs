
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub timestamp: i64,
}

#[derive(Debug)]
pub enum ProcessingError {
    InvalidValue,
    InvalidTimestamp,
    SerializationError(String),
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessingError::InvalidValue => write!(f, "Invalid data value"),
            ProcessingError::InvalidTimestamp => write!(f, "Invalid timestamp"),
            ProcessingError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
        }
    }
}

impl Error for ProcessingError {}

pub struct DataProcessor {
    threshold: f64,
}

impl DataProcessor {
    pub fn new(threshold: f64) -> Self {
        DataProcessor { threshold }
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if record.value.is_nan() || record.value.is_infinite() {
            return Err(ProcessingError::InvalidValue);
        }

        if record.timestamp < 0 {
            return Err(ProcessingError::InvalidTimestamp);
        }

        Ok(())
    }

    pub fn process_record(&self, record: &DataRecord) -> Result<DataRecord, ProcessingError> {
        self.validate_record(record)?;

        let processed_value = if record.value > self.threshold {
            record.value * 0.9
        } else {
            record.value * 1.1
        };

        Ok(DataRecord {
            id: record.id,
            value: processed_value,
            timestamp: record.timestamp,
        })
    }

    pub fn serialize_record(&self, record: &DataRecord) -> Result<String, ProcessingError> {
        serde_json::to_string(record)
            .map_err(|e| ProcessingError::SerializationError(e.to_string()))
    }

    pub fn deserialize_record(&self, data: &str) -> Result<DataRecord, ProcessingError> {
        serde_json::from_str(data)
            .map_err(|e| ProcessingError::SerializationError(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_valid_record() {
        let processor = DataProcessor::new(100.0);
        let record = DataRecord {
            id: 1,
            value: 50.0,
            timestamp: 1234567890,
        };
        assert!(processor.validate_record(&record).is_ok());
    }

    #[test]
    fn test_validation_invalid_value() {
        let processor = DataProcessor::new(100.0);
        let record = DataRecord {
            id: 1,
            value: f64::NAN,
            timestamp: 1234567890,
        };
        assert!(processor.validate_record(&record).is_err());
    }

    #[test]
    fn test_processing_above_threshold() {
        let processor = DataProcessor::new(100.0);
        let record = DataRecord {
            id: 1,
            value: 150.0,
            timestamp: 1234567890,
        };
        let processed = processor.process_record(&record).unwrap();
        assert_eq!(processed.value, 135.0);
    }

    #[test]
    fn test_serialization_roundtrip() {
        let processor = DataProcessor::new(100.0);
        let original = DataRecord {
            id: 42,
            value: 75.5,
            timestamp: 987654321,
        };
        
        let serialized = processor.serialize_record(&original).unwrap();
        let deserialized = processor.deserialize_record(&serialized).unwrap();
        
        assert_eq!(original.id, deserialized.id);
        assert_eq!(original.value, deserialized.value);
        assert_eq!(original.timestamp, deserialized.timestamp);
    }
}use csv::Reader;
use serde::Deserialize;
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

pub fn process_csv_file(path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(path)?;
    let mut reader = Reader::from_reader(file);
    let mut records = Vec::new();

    for result in reader.deserialize() {
        let record: Record = result?;
        if record.value < 0.0 {
            return Err(format!("Invalid value in record ID {}", record.id).into());
        }
        records.push(record);
    }

    Ok(records)
}

pub fn calculate_statistics(records: &[Record]) -> (f64, f64, f64) {
    let count = records.len() as f64;
    if count == 0.0 {
        return (0.0, 0.0, 0.0);
    }

    let sum: f64 = records.iter().map(|r| r.value).sum();
    let mean = sum / count;
    
    let variance: f64 = records.iter()
        .map(|r| (r.value - mean).powi(2))
        .sum::<f64>() / count;
    
    let std_dev = variance.sqrt();
    
    (sum, mean, std_dev)
}

pub fn filter_by_category(records: Vec<Record>, category: &str) -> Vec<Record> {
    records.into_iter()
        .filter(|r| r.category == category)
        .collect()
}use csv::Reader;
use serde::Deserialize;
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

pub fn process_data_file(file_path: &str, category_filter: Option<&str>) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let mut reader = Reader::from_reader(file);
    let mut filtered_records = Vec::new();

    for result in reader.deserialize() {
        let record: Record = result?;
        
        match category_filter {
            Some(filter) if record.category == filter => filtered_records.push(record),
            None => filtered_records.push(record),
            _ => continue,
        }
    }

    Ok(filtered_records)
}

pub fn calculate_statistics(records: &[Record]) -> (f64, f64, f64) {
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
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_process_data_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,category").unwrap();
        writeln!(temp_file, "1,ItemA,10.5,Category1").unwrap();
        writeln!(temp_file, "2,ItemB,20.3,Category2").unwrap();
        
        let records = process_data_file(temp_file.path().to_str().unwrap(), None).unwrap();
        assert_eq!(records.len(), 2);
    }

    #[test]
    fn test_calculate_statistics() {
        let records = vec![
            Record { id: 1, name: "Test1".to_string(), value: 10.0, category: "Cat1".to_string() },
            Record { id: 2, name: "Test2".to_string(), value: 20.0, category: "Cat1".to_string() },
        ];
        
        let (mean, variance, std_dev) = calculate_statistics(&records);
        assert_eq!(mean, 15.0);
        assert_eq!(variance, 25.0);
        assert_eq!(std_dev, 5.0);
    }
}
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRecord {
    pub id: u64,
    pub timestamp: i64,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug)]
pub enum ProcessingError {
    InvalidData(String),
    TransformationFailed(String),
    ValidationError(String),
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessingError::InvalidData(msg) => write!(f, "Invalid data: {}", msg),
            ProcessingError::TransformationFailed(msg) => write!(f, "Transformation failed: {}", msg),
            ProcessingError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl Error for ProcessingError {}

pub struct DataProcessor {
    config: ProcessingConfig,
}

#[derive(Debug, Clone)]
pub struct ProcessingConfig {
    pub max_values: usize,
    pub require_timestamp: bool,
    pub allowed_metadata_keys: Vec<String>,
}

impl Default for ProcessingConfig {
    fn default() -> Self {
        ProcessingConfig {
            max_values: 100,
            require_timestamp: true,
            allowed_metadata_keys: vec![],
        }
    }
}

impl DataProcessor {
    pub fn new(config: ProcessingConfig) -> Self {
        DataProcessor { config }
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if record.values.len() > self.config.max_values {
            return Err(ProcessingError::ValidationError(format!(
                "Too many values: {} > {}",
                record.values.len(),
                self.config.max_values
            )));
        }

        if self.config.require_timestamp && record.timestamp <= 0 {
            return Err(ProcessingError::ValidationError(
                "Invalid timestamp".to_string(),
            ));
        }

        if !self.config.allowed_metadata_keys.is_empty() {
            for key in record.metadata.keys() {
                if !self.config.allowed_metadata_keys.contains(key) {
                    return Err(ProcessingError::ValidationError(format!(
                        "Disallowed metadata key: {}",
                        key
                    )));
                }
            }
        }

        Ok(())
    }

    pub fn transform_record(
        &self,
        record: &DataRecord,
        transformation: &Transformation,
    ) -> Result<DataRecord, ProcessingError> {
        let mut transformed = record.clone();

        match transformation {
            Transformation::Normalize => {
                if transformed.values.is_empty() {
                    return Err(ProcessingError::TransformationFailed(
                        "No values to normalize".to_string(),
                    ));
                }

                let sum: f64 = transformed.values.iter().sum();
                if sum == 0.0 {
                    return Err(ProcessingError::TransformationFailed(
                        "Cannot normalize zero sum".to_string(),
                    ));
                }

                for value in transformed.values.iter_mut() {
                    *value /= sum;
                }
            }
            Transformation::Scale(factor) => {
                for value in transformed.values.iter_mut() {
                    *value *= factor;
                }
            }
            Transformation::AddMetadata(key, value) => {
                transformed.metadata.insert(key.clone(), value.clone());
            }
        }

        Ok(transformed)
    }

    pub fn batch_process(
        &self,
        records: Vec<DataRecord>,
        transformation: Option<Transformation>,
    ) -> Result<Vec<DataRecord>, ProcessingError> {
        let mut results = Vec::with_capacity(records.len());

        for record in records {
            self.validate_record(&record)?;

            let processed_record = if let Some(ref trans) = transformation {
                self.transform_record(&record, trans)?
            } else {
                record
            };

            results.push(processed_record);
        }

        Ok(results)
    }
}

#[derive(Debug, Clone)]
pub enum Transformation {
    Normalize,
    Scale(f64),
    AddMetadata(String, String),
}

pub fn create_sample_record() -> DataRecord {
    let mut metadata = HashMap::new();
    metadata.insert("source".to_string(), "sensor_001".to_string());
    metadata.insert("unit".to_string(), "celsius".to_string());

    DataRecord {
        id: 1,
        timestamp: 1672531200,
        values: vec![20.5, 21.0, 19.8, 22.3, 20.9],
        metadata,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_success() {
        let processor = DataProcessor::new(ProcessingConfig::default());
        let record = create_sample_record();
        assert!(processor.validate_record(&record).is_ok());
    }

    #[test]
    fn test_validation_too_many_values() {
        let config = ProcessingConfig {
            max_values: 3,
            ..Default::default()
        };
        let processor = DataProcessor::new(config);
        let record = create_sample_record();
        assert!(processor.validate_record(&record).is_err());
    }

    #[test]
    fn test_normalize_transformation() {
        let processor = DataProcessor::new(ProcessingConfig::default());
        let record = create_sample_record();
        let result = processor.transform_record(&record, &Transformation::Normalize);
        assert!(result.is_ok());

        let normalized = result.unwrap();
        let sum: f64 = normalized.values.iter().sum();
        assert!((sum - 1.0).abs() < 0.0001);
    }

    #[test]
    fn test_batch_processing() {
        let processor = DataProcessor::new(ProcessingConfig::default());
        let records = vec![create_sample_record(), create_sample_record()];
        let result = processor.batch_process(records, Some(Transformation::Scale(2.0)));
        assert!(result.is_ok());

        let processed = result.unwrap();
        assert_eq!(processed.len(), 2);
        assert!((processed[0].values[0] - 41.0).abs() < 0.0001);
    }
}