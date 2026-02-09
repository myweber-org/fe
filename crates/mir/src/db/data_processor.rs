
use std::collections::HashMap;

pub struct DataProcessor {
    validators: HashMap<String, Box<dyn Fn(&str) -> bool>>,
    transformers: HashMap<String, Box<dyn Fn(String) -> String>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            validators: HashMap::new(),
            transformers: HashMap::new(),
        }
    }

    pub fn register_validator(&mut self, name: &str, validator: Box<dyn Fn(&str) -> bool>) {
        self.validators.insert(name.to_string(), validator);
    }

    pub fn register_transformer(&mut self, name: &str, transformer: Box<dyn Fn(String) -> String>) {
        self.transformers.insert(name.to_string(), transformer);
    }

    pub fn validate(&self, name: &str, data: &str) -> bool {
        self.validators
            .get(name)
            .map(|validator| validator(data))
            .unwrap_or(false)
    }

    pub fn transform(&self, name: &str, data: String) -> Option<String> {
        self.transformers
            .get(name)
            .map(|transformer| transformer(data))
    }

    pub fn process_pipeline(&self, data: String, steps: Vec<(&str, &str)>) -> Result<String, String> {
        let mut current_data = data;

        for (operation, name) in steps {
            match operation {
                "validate" => {
                    if !self.validate(name, &current_data) {
                        return Err(format!("Validation '{}' failed for data: {}", name, current_data));
                    }
                }
                "transform" => {
                    current_data = self.transform(name, current_data)
                        .ok_or_else(|| format!("Transformer '{}' not found", name))?;
                }
                _ => return Err(format!("Unknown operation: {}", operation)),
            }
        }

        Ok(current_data)
    }
}

pub fn create_default_processor() -> DataProcessor {
    let mut processor = DataProcessor::new();

    processor.register_validator("non_empty", Box::new(|s| !s.trim().is_empty()));
    processor.register_validator("is_numeric", Box::new(|s| s.chars().all(|c| c.is_ascii_digit())));
    processor.register_validator("is_alpha", Box::new(|s| s.chars().all(|c| c.is_ascii_alphabetic())));

    processor.register_transformer("to_uppercase", Box::new(|s| s.to_uppercase()));
    processor.register_transformer("to_lowercase", Box::new(|s| s.to_lowercase()));
    processor.register_transformer("trim_spaces", Box::new(|s| s.trim().to_string()));
    processor.register_transformer("reverse_string", Box::new(|s| s.chars().rev().collect()));

    processor
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation() {
        let processor = create_default_processor();
        assert!(processor.validate("non_empty", "test"));
        assert!(!processor.validate("non_empty", "   "));
        assert!(processor.validate("is_numeric", "12345"));
        assert!(!processor.validate("is_numeric", "123a"));
    }

    #[test]
    fn test_transformation() {
        let processor = create_default_processor();
        assert_eq!(processor.transform("to_uppercase", "hello".to_string()), Some("HELLO".to_string()));
        assert_eq!(processor.transform("reverse_string", "abc".to_string()), Some("cba".to_string()));
    }

    #[test]
    fn test_pipeline() {
        let processor = create_default_processor();
        let steps = vec![
            ("validate", "non_empty"),
            ("transform", "to_uppercase"),
            ("transform", "reverse_string"),
        ];
        
        let result = processor.process_pipeline("hello".to_string(), steps);
        assert_eq!(result, Ok("OLLEH".to_string()));
        
        let invalid_result = processor.process_pipeline("   ".to_string(), vec![("validate", "non_empty")]);
        assert!(invalid_result.is_err());
    }
}use csv::{ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    active: bool,
}

struct DataProcessor {
    records: Vec<Record>,
}

impl DataProcessor {
    fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    fn load_from_csv(&mut self, path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
        let mut rdr = ReaderBuilder::new().has_headers(true).from_reader(file);

        for result in rdr.deserialize() {
            let record: Record = result?;
            self.records.push(record);
        }

        Ok(())
    }

    fn filter_by_value(&self, threshold: f64) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|record| record.value > threshold && record.active)
            .collect()
    }

    fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }

        let sum: f64 = self.records.iter().map(|record| record.value).sum();
        Some(sum / self.records.len() as f64)
    }

    fn save_filtered_to_csv(&self, path: &str, threshold: f64) -> Result<(), Box<dyn Error>> {
        let filtered = self.filter_by_value(threshold);
        let file = File::create(path)?;
        let mut wtr = WriterBuilder::new().has_headers(true).from_writer(file);

        for record in filtered {
            wtr.serialize(record)?;
        }

        wtr.flush()?;
        Ok(())
    }
}

fn process_data_file(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let mut processor = DataProcessor::new();
    processor.load_from_csv(input_path)?;

    if let Some(avg) = processor.calculate_average() {
        println!("Average value: {:.2}", avg);
        let threshold = avg * 0.8;
        processor.save_filtered_to_csv(output_path, threshold)?;
        println!("Filtered data saved to {}", output_path);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processing() {
        let csv_data = "id,name,value,active\n1,ItemA,10.5,true\n2,ItemB,5.2,false\n3,ItemC,15.8,true\n";
        
        let mut temp_input = NamedTempFile::new().unwrap();
        std::fs::write(temp_input.path(), csv_data).unwrap();
        
        let temp_output = NamedTempFile::new().unwrap();
        
        let result = process_data_file(
            temp_input.path().to_str().unwrap(),
            temp_output.path().to_str().unwrap()
        );
        
        assert!(result.is_ok());
        
        let output_content = std::fs::read_to_string(temp_output.path()).unwrap();
        assert!(output_content.contains("ItemA"));
        assert!(!output_content.contains("ItemB"));
        assert!(output_content.contains("ItemC"));
    }
}
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DataRecord {
    pub id: u64,
    pub timestamp: i64,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, PartialEq)]
pub enum ValidationError {
    InvalidId,
    InvalidTimestamp,
    EmptyValues,
    MetadataTooLarge,
}

pub struct DataProcessor {
    max_metadata_size: usize,
    min_values_count: usize,
}

impl DataProcessor {
    pub fn new(max_metadata_size: usize, min_values_count: usize) -> Self {
        DataProcessor {
            max_metadata_size,
            min_values_count,
        }
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), ValidationError> {
        if record.id == 0 {
            return Err(ValidationError::InvalidId);
        }

        if record.timestamp <= 0 {
            return Err(ValidationError::InvalidTimestamp);
        }

        if record.values.len() < self.min_values_count {
            return Err(ValidationError::EmptyValues);
        }

        let total_metadata_size: usize = record.metadata
            .iter()
            .map(|(k, v)| k.len() + v.len())
            .sum();

        if total_metadata_size > self.max_metadata_size {
            return Err(ValidationError::MetadataTooLarge);
        }

        Ok(())
    }

    pub fn transform_values(&self, record: &mut DataRecord, transform_fn: fn(f64) -> f64) {
        record.values = record.values
            .iter()
            .map(|&value| transform_fn(value))
            .collect();
    }

    pub fn calculate_statistics(&self, record: &DataRecord) -> HashMap<String, f64> {
        let mut stats = HashMap::new();

        if record.values.is_empty() {
            return stats;
        }

        let sum: f64 = record.values.iter().sum();
        let count = record.values.len() as f64;
        let mean = sum / count;

        let variance: f64 = record.values
            .iter()
            .map(|&value| (value - mean).powi(2))
            .sum::<f64>() / count;

        let min = record.values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = record.values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

        stats.insert("mean".to_string(), mean);
        stats.insert("variance".to_string(), variance);
        stats.insert("min".to_string(), min);
        stats.insert("max".to_string(), max);
        stats.insert("count".to_string(), count);
        stats.insert("sum".to_string(), sum);

        stats
    }

    pub fn merge_records(&self, records: Vec<DataRecord>) -> Result<DataRecord, Box<dyn Error>> {
        if records.is_empty() {
            return Err("No records to merge".into());
        }

        let first_record = &records[0];
        let mut merged_values = Vec::new();
        let mut merged_metadata = first_record.metadata.clone();

        for record in &records {
            self.validate_record(record)?;
            merged_values.extend_from_slice(&record.values);

            for (key, value) in &record.metadata {
                merged_metadata.entry(key.clone())
                    .and_modify(|v| *v = format!("{};{}", v, value))
                    .or_insert(value.clone());
            }
        }

        Ok(DataRecord {
            id: records.iter().map(|r| r.id).max().unwrap_or(0),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs() as i64,
            values: merged_values,
            metadata: merged_metadata,
        })
    }
}

pub fn normalize_value(value: f64) -> f64 {
    if value == 0.0 {
        0.0
    } else {
        value.signum() * value.abs().ln()
    }
}

pub fn scale_value(factor: f64) -> impl Fn(f64) -> f64 {
    move |value| value * factor
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation() {
        let processor = DataProcessor::new(100, 2);
        let mut record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: vec![1.0, 2.0, 3.0],
            metadata: HashMap::new(),
        };

        assert!(processor.validate_record(&record).is_ok());

        record.id = 0;
        assert_eq!(processor.validate_record(&record), Err(ValidationError::InvalidId));

        record.id = 1;
        record.timestamp = -1;
        assert_eq!(processor.validate_record(&record), Err(ValidationError::InvalidTimestamp));

        record.timestamp = 1234567890;
        record.values.clear();
        assert_eq!(processor.validate_record(&record), Err(ValidationError::EmptyValues));
    }

    #[test]
    fn test_statistics() {
        let processor = DataProcessor::new(100, 1);
        let record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: vec![1.0, 2.0, 3.0, 4.0, 5.0],
            metadata: HashMap::new(),
        };

        let stats = processor.calculate_statistics(&record);
        assert_eq!(stats.get("mean"), Some(&3.0));
        assert_eq!(stats.get("sum"), Some(&15.0));
        assert_eq!(stats.get("count"), Some(&5.0));
    }

    #[test]
    fn test_value_transformation() {
        let processor = DataProcessor::new(100, 1);
        let mut record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: vec![1.0, 2.0, 3.0],
            metadata: HashMap::new(),
        };

        processor.transform_values(&mut record, |x| x * 2.0);
        assert_eq!(record.values, vec![2.0, 4.0, 6.0]);
    }
}use csv::Reader;
use serde::Deserialize;
use std::error::Error;

#[derive(Debug, Deserialize)]
pub struct Record {
    id: u32,
    name: String,
    value: f64,
}

pub fn process_csv_data(input_path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let mut reader = Reader::from_path(input_path)?;
    let mut records = Vec::new();

    for result in reader.deserialize() {
        let record: Record = result?;
        if record.value >= 0.0 {
            records.push(record);
        }
    }

    Ok(records)
}

pub fn calculate_average(records: &[Record]) -> Option<f64> {
    if records.is_empty() {
        return None;
    }
    
    let sum: f64 = records.iter().map(|r| r.value).sum();
    Some(sum / records.len() as f64)
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct DataRecord {
    id: u32,
    value: f64,
    category: String,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: String) -> Result<Self, String> {
        if value < 0.0 {
            return Err(format!("Invalid value: {}", value));
        }
        if category.is_empty() {
            return Err("Category cannot be empty".to_string());
        }
        Ok(Self { id, value, category })
    }

    pub fn calculate_score(&self) -> f64 {
        self.value * (self.category.len() as f64)
    }
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        Self { records: Vec::new() }
    }

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<usize, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut count = 0;

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            if line_num == 0 {
                continue;
            }

            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() != 3 {
                continue;
            }

            let id = parts[0].parse::<u32>()?;
            let value = parts[1].parse::<f64>()?;
            let category = parts[2].to_string();

            match DataRecord::new(id, value, category) {
                Ok(record) => {
                    self.records.push(record);
                    count += 1;
                }
                Err(e) => eprintln!("Skipping invalid record at line {}: {}", line_num + 1, e),
            }
        }

        Ok(count)
    }

    pub fn total_score(&self) -> f64 {
        self.records.iter().map(|r| r.calculate_score()).sum()
    }

    pub fn average_value(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }
        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|r| r.category == category)
            .collect()
    }

    pub fn record_count(&self) -> usize {
        self.records.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_record_creation() {
        let record = DataRecord::new(1, 42.5, "test".to_string()).unwrap();
        assert_eq!(record.id, 1);
        assert_eq!(record.value, 42.5);
        assert_eq!(record.category, "test");
    }

    #[test]
    fn test_invalid_data_record() {
        let result = DataRecord::new(1, -5.0, "test".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_calculate_score() {
        let record = DataRecord::new(1, 10.0, "abc".to_string()).unwrap();
        assert_eq!(record.calculate_score(), 30.0);
    }

    #[test]
    fn test_load_csv() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category").unwrap();
        writeln!(temp_file, "1,10.5,alpha").unwrap();
        writeln!(temp_file, "2,20.0,beta").unwrap();
        writeln!(temp_file, "3,-5.0,gamma").unwrap();

        let mut processor = DataProcessor::new();
        let result = processor.load_from_csv(temp_file.path());
        assert!(result.is_ok());
        assert_eq!(processor.record_count(), 2);
        assert_eq!(processor.total_score(), 10.5 * 5.0 + 20.0 * 4.0);
    }

    #[test]
    fn test_average_value() {
        let mut processor = DataProcessor::new();
        processor.records.push(DataRecord::new(1, 10.0, "a".to_string()).unwrap());
        processor.records.push(DataRecord::new(2, 20.0, "b".to_string()).unwrap());
        assert_eq!(processor.average_value(), Some(15.0));
    }

    #[test]
    fn test_filter_by_category() {
        let mut processor = DataProcessor::new();
        processor.records.push(DataRecord::new(1, 10.0, "alpha".to_string()).unwrap());
        processor.records.push(DataRecord::new(2, 20.0, "beta".to_string()).unwrap());
        processor.records.push(DataRecord::new(3, 30.0, "alpha".to_string()).unwrap());

        let filtered = processor.filter_by_category("alpha");
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0].id, 1);
        assert_eq!(filtered[1].id, 3);
    }
}
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Serialize, Deserialize)]
pub struct DataRecord {
    pub id: u64,
    pub value: f64,
    pub timestamp: i64,
}

#[derive(Debug, Error)]
pub enum ProcessingError {
    #[error("Invalid data value: {0}")]
    InvalidValue(f64),
    #[error("Timestamp out of range: {0}")]
    InvalidTimestamp(i64),
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

pub struct DataProcessor {
    min_value: f64,
    max_value: f64,
}

impl DataProcessor {
    pub fn new(min_value: f64, max_value: f64) -> Self {
        DataProcessor { min_value, max_value }
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if record.value < self.min_value || record.value > self.max_value {
            return Err(ProcessingError::InvalidValue(record.value));
        }

        if record.timestamp < 0 {
            return Err(ProcessingError::InvalidTimestamp(record.timestamp));
        }

        Ok(())
    }

    pub fn transform_record(&self, record: DataRecord) -> Result<DataRecord, ProcessingError> {
        self.validate_record(&record)?;

        let transformed = DataRecord {
            id: record.id,
            value: record.value * 2.0,
            timestamp: record.timestamp + 1,
        };

        Ok(transformed)
    }

    pub fn process_batch(
        &self,
        records: Vec<DataRecord>,
    ) -> Result<Vec<DataRecord>, ProcessingError> {
        let mut results = Vec::with_capacity(records.len());

        for record in records {
            match self.transform_record(record) {
                Ok(transformed) => results.push(transformed),
                Err(e) => return Err(e),
            }
        }

        Ok(results)
    }

    pub fn serialize_records(&self, records: &[DataRecord]) -> Result<String, ProcessingError> {
        serde_json::to_string(records)
            .map_err(|e| ProcessingError::SerializationError(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_record_processing() {
        let processor = DataProcessor::new(0.0, 100.0);
        let record = DataRecord {
            id: 1,
            value: 50.0,
            timestamp: 1000,
        };

        let result = processor.transform_record(record);
        assert!(result.is_ok());

        let transformed = result.unwrap();
        assert_eq!(transformed.value, 100.0);
        assert_eq!(transformed.timestamp, 1001);
    }

    #[test]
    fn test_invalid_value() {
        let processor = DataProcessor::new(0.0, 100.0);
        let record = DataRecord {
            id: 1,
            value: 150.0,
            timestamp: 1000,
        };

        let result = processor.transform_record(record);
        assert!(matches!(result, Err(ProcessingError::InvalidValue(150.0))));
    }

    #[test]
    fn test_batch_processing() {
        let processor = DataProcessor::new(0.0, 100.0);
        let records = vec![
            DataRecord {
                id: 1,
                value: 10.0,
                timestamp: 1000,
            },
            DataRecord {
                id: 2,
                value: 20.0,
                timestamp: 2000,
            },
        ];

        let result = processor.process_batch(records);
        assert!(result.is_ok());

        let processed = result.unwrap();
        assert_eq!(processed.len(), 2);
        assert_eq!(processed[0].value, 20.0);
        assert_eq!(processed[1].value, 40.0);
    }
}