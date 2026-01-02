
use csv::Reader;
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

pub fn process_data_file(file_path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let mut reader = Reader::from_reader(file);
    let mut records = Vec::new();

    for result in reader.deserialize() {
        let record: Record = result?;
        validate_record(&record)?;
        records.push(record);
    }

    Ok(records)
}

fn validate_record(record: &Record) -> Result<(), String> {
    if record.name.is_empty() {
        return Err("Name cannot be empty".to_string());
    }
    if record.value < 0.0 {
        return Err("Value cannot be negative".to_string());
    }
    if !["A", "B", "C"].contains(&record.category.as_str()) {
        return Err("Invalid category".to_string());
    }
    Ok(())
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

    (mean, variance, std_dev)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_process_valid_data() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,category").unwrap();
        writeln!(temp_file, "1,Test1,10.5,A").unwrap();
        writeln!(temp_file, "2,Test2,20.0,B").unwrap();

        let result = process_data_file(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 2);
    }

    #[test]
    fn test_statistics_calculation() {
        let records = vec![
            Record { id: 1, name: "A".to_string(), value: 10.0, category: "A".to_string() },
            Record { id: 2, name: "B".to_string(), value: 20.0, category: "B".to_string() },
            Record { id: 3, name: "C".to_string(), value: 30.0, category: "C".to_string() },
        ];

        let (mean, variance, std_dev) = calculate_statistics(&records);
        assert_eq!(mean, 20.0);
        assert_eq!(variance, 66.66666666666667);
        assert_eq!(std_dev, 8.16496580927726);
    }
}
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Serialize, Deserialize, Clone)]
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
    validation_rules: Vec<Box<dyn Fn(&DataRecord) -> Result<(), ProcessingError>>>,
    transformation_pipeline: Vec<Box<dyn Fn(DataRecord) -> Result<DataRecord, ProcessingError>>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            validation_rules: Vec::new(),
            transformation_pipeline: Vec::new(),
        }
    }

    pub fn add_validation_rule<F>(&mut self, rule: F)
    where
        F: Fn(&DataRecord) -> Result<(), ProcessingError> + 'static,
    {
        self.validation_rules.push(Box::new(rule));
    }

    pub fn add_transformation<F>(&mut self, transform: F)
    where
        F: Fn(DataRecord) -> Result<DataRecord, ProcessingError> + 'static,
    {
        self.transformation_pipeline.push(Box::new(transform));
    }

    pub fn process(&self, mut record: DataRecord) -> Result<DataRecord, ProcessingError> {
        for rule in &self.validation_rules {
            rule(&record)?;
        }

        for transform in &self.transformation_pipeline {
            record = transform(record)?;
        }

        Ok(record)
    }

    pub fn batch_process(&self, records: Vec<DataRecord>) -> Vec<Result<DataRecord, ProcessingError>> {
        records.into_iter().map(|record| self.process(record)).collect()
    }
}

pub fn validate_timestamp(record: &DataRecord) -> Result<(), ProcessingError> {
    if record.timestamp < 0 {
        return Err(ProcessingError::ValidationError(
            "Timestamp cannot be negative".to_string(),
        ));
    }
    Ok(())
}

pub fn validate_values(record: &DataRecord) -> Result<(), ProcessingError> {
    if record.values.is_empty() {
        return Err(ProcessingError::ValidationError(
            "Values vector cannot be empty".to_string(),
        ));
    }

    for value in &record.values {
        if value.is_nan() || value.is_infinite() {
            return Err(ProcessingError::ValidationError(
                "Values contain NaN or infinite numbers".to_string(),
            ));
        }
    }
    Ok(())
}

pub fn normalize_values(record: DataRecord) -> Result<DataRecord, ProcessingError> {
    if record.values.is_empty() {
        return Err(ProcessingError::TransformationFailed(
            "Cannot normalize empty values vector".to_string(),
        ));
    }

    let sum: f64 = record.values.iter().sum();
    if sum == 0.0 {
        return Ok(record);
    }

    let normalized_values: Vec<f64> = record.values.iter().map(|v| v / sum).collect();

    Ok(DataRecord {
        values: normalized_values,
        ..record
    })
}

pub fn add_processing_metadata(record: DataRecord) -> Result<DataRecord, ProcessingError> {
    let mut new_metadata = record.metadata.clone();
    new_metadata.insert(
        "processed_at".to_string(),
        chrono::Utc::now().timestamp().to_string(),
    );
    new_metadata.insert(
        "values_count".to_string(),
        record.values.len().to_string(),
    );

    Ok(DataRecord {
        metadata: new_metadata,
        ..record
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_record() -> DataRecord {
        DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: vec![1.0, 2.0, 3.0],
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn test_validation_rules() {
        let mut processor = DataProcessor::new();
        processor.add_validation_rule(validate_timestamp);
        processor.add_validation_rule(validate_values);

        let valid_record = create_test_record();
        assert!(processor.process(valid_record).is_ok());

        let mut invalid_record = create_test_record();
        invalid_record.timestamp = -1;
        assert!(processor.process(invalid_record).is_err());
    }

    #[test]
    fn test_transformation_pipeline() {
        let mut processor = DataProcessor::new();
        processor.add_transformation(normalize_values);
        processor.add_transformation(add_processing_metadata);

        let record = create_test_record();
        let result = processor.process(record).unwrap();

        let sum: f64 = result.values.iter().sum();
        assert!((sum - 1.0).abs() < 0.0001);
        assert!(result.metadata.contains_key("processed_at"));
        assert!(result.metadata.contains_key("values_count"));
    }

    #[test]
    fn test_batch_processing() {
        let processor = DataProcessor::new();
        let records = vec![create_test_record(), create_test_record()];
        let results = processor.batch_process(records);

        assert_eq!(results.len(), 2);
        for result in results {
            assert!(result.is_ok());
        }
    }
}