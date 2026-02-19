
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
    validation_threshold: f64,
    transformation_factor: f64,
}

impl DataProcessor {
    pub fn new(validation_threshold: f64, transformation_factor: f64) -> Self {
        DataProcessor {
            validation_threshold,
            transformation_factor,
        }
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if record.values.is_empty() {
            return Err(ProcessingError::InvalidData("Empty values vector".to_string()));
        }

        for value in &record.values {
            if value.is_nan() || value.is_infinite() {
                return Err(ProcessingError::InvalidData(
                    "Invalid numeric value detected".to_string(),
                ));
            }
        }

        let sum: f64 = record.values.iter().sum();
        if sum.abs() > self.validation_threshold {
            return Err(ProcessingError::ValidationError(
                format!("Sum {} exceeds threshold {}", sum, self.validation_threshold),
            ));
        }

        Ok(())
    }

    pub fn transform_record(&self, record: &mut DataRecord) -> Result<(), ProcessingError> {
        self.validate_record(record)?;

        for value in &mut record.values {
            *value *= self.transformation_factor;
            
            if value.is_nan() || value.is_infinite() {
                return Err(ProcessingError::TransformationFailed(
                    "Transformation produced invalid value".to_string(),
                ));
            }
        }

        record.metadata.insert(
            "processed".to_string(),
            "true".to_string(),
        );
        record.metadata.insert(
            "transformation_factor".to_string(),
            self.transformation_factor.to_string(),
        );

        Ok(())
    }

    pub fn batch_process(
        &self,
        records: &mut [DataRecord],
    ) -> Result<Vec<DataRecord>, ProcessingError> {
        let mut processed_records = Vec::with_capacity(records.len());
        let mut errors = Vec::new();

        for record in records.iter_mut() {
            match self.transform_record(record) {
                Ok(_) => processed_records.push(record.clone()),
                Err(e) => errors.push((record.id, e.to_string())),
            }
        }

        if !errors.is_empty() {
            let error_msg = errors
                .iter()
                .map(|(id, msg)| format!("Record {}: {}", id, msg))
                .collect::<Vec<String>>()
                .join("; ");
            return Err(ProcessingError::ProcessingError(error_msg));
        }

        Ok(processed_records)
    }

    pub fn calculate_statistics(&self, records: &[DataRecord]) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        
        if records.is_empty() {
            return stats;
        }

        let all_values: Vec<f64> = records
            .iter()
            .flat_map(|r| r.values.clone())
            .collect();

        let count = all_values.len() as f64;
        let sum: f64 = all_values.iter().sum();
        let mean = sum / count;

        let variance: f64 = all_values
            .iter()
            .map(|v| (v - mean).powi(2))
            .sum::<f64>()
            / count;

        stats.insert("count".to_string(), count);
        stats.insert("sum".to_string(), sum);
        stats.insert("mean".to_string(), mean);
        stats.insert("variance".to_string(), variance);
        stats.insert("std_dev".to_string(), variance.sqrt());

        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_success() {
        let processor = DataProcessor::new(1000.0, 2.0);
        let record = DataRecord {
            id: 1,
            values: vec![10.0, 20.0, 30.0],
            metadata: HashMap::new(),
        };

        assert!(processor.validate_record(&record).is_ok());
    }

    #[test]
    fn test_validation_threshold_exceeded() {
        let processor = DataProcessor::new(50.0, 2.0);
        let record = DataRecord {
            id: 1,
            values: vec![100.0, 200.0, 300.0],
            metadata: HashMap::new(),
        };

        assert!(processor.validate_record(&record).is_err());
    }

    #[test]
    fn test_transform_record() {
        let processor = DataProcessor::new(1000.0, 2.0);
        let mut record = DataRecord {
            id: 1,
            values: vec![10.0, 20.0, 30.0],
            metadata: HashMap::new(),
        };

        assert!(processor.transform_record(&mut record).is_ok());
        assert_eq!(record.values, vec![20.0, 40.0, 60.0]);
        assert_eq!(record.metadata.get("processed"), Some(&"true".to_string()));
    }
}use std::collections::HashMap;
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
    MetadataKeyMissing(String),
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataError::InvalidId => write!(f, "Invalid record ID"),
            DataError::EmptyValues => write!(f, "Record values cannot be empty"),
            DataError::ValueOutOfRange(val) => write!(f, "Value {} is out of acceptable range", val),
            DataError::MetadataKeyMissing(key) => write!(f, "Required metadata key '{}' is missing", key),
        }
    }
}

impl Error for DataError {}

impl DataRecord {
    pub fn new(id: u32, values: Vec<f64>, metadata: HashMap<String, String>) -> Result<Self, DataError> {
        if id == 0 {
            return Err(DataError::InvalidId);
        }
        
        if values.is_empty() {
            return Err(DataError::EmptyValues);
        }
        
        for &value in &values {
            if !value.is_finite() {
                return Err(DataError::ValueOutOfRange(value));
            }
        }
        
        Ok(Self { id, values, metadata })
    }
    
    pub fn validate_metadata(&self, required_keys: &[&str]) -> Result<(), DataError> {
        for key in required_keys {
            if !self.metadata.contains_key(*key) {
                return Err(DataError::MetadataKeyMissing(key.to_string()));
            }
        }
        Ok(())
    }
    
    pub fn calculate_statistics(&self) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        
        if self.values.is_empty() {
            return stats;
        }
        
        let sum: f64 = self.values.iter().sum();
        let count = self.values.len() as f64;
        let mean = sum / count;
        
        let variance: f64 = self.values.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / count;
        
        stats.insert("mean".to_string(), mean);
        stats.insert("sum".to_string(), sum);
        stats.insert("count".to_string(), count);
        stats.insert("variance".to_string(), variance);
        
        stats
    }
    
    pub fn normalize_values(&mut self) {
        let stats = self.calculate_statistics();
        if let Some(&mean) = stats.get("mean") {
            if let Some(&variance) = stats.get("variance") {
                let std_dev = variance.sqrt();
                if std_dev > 0.0 {
                    for value in &mut self.values {
                        *value = (*value - mean) / std_dev;
                    }
                }
            }
        }
    }
}

pub fn process_records(records: &mut [DataRecord], required_metadata: &[&str]) -> Result<Vec<HashMap<String, f64>>, DataError> {
    let mut results = Vec::new();
    
    for record in records {
        record.validate_metadata(required_metadata)?;
        record.normalize_values();
        results.push(record.calculate_statistics());
    }
    
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_valid_record_creation() {
        let metadata = HashMap::from([
            ("source".to_string(), "sensor_a".to_string()),
            ("timestamp".to_string(), "2024-01-15T10:30:00Z".to_string()),
        ]);
        
        let record = DataRecord::new(
            1,
            vec![1.0, 2.0, 3.0, 4.0, 5.0],
            metadata,
        ).unwrap();
        
        assert_eq!(record.id, 1);
        assert_eq!(record.values.len(), 5);
    }
    
    #[test]
    fn test_invalid_id() {
        let result = DataRecord::new(0, vec![1.0], HashMap::new());
        assert!(matches!(result, Err(DataError::InvalidId)));
    }
    
    #[test]
    fn test_statistics_calculation() {
        let record = DataRecord::new(
            1,
            vec![1.0, 2.0, 3.0, 4.0, 5.0],
            HashMap::new(),
        ).unwrap();
        
        let stats = record.calculate_statistics();
        assert_eq!(stats.get("mean"), Some(&3.0));
        assert_eq!(stats.get("sum"), Some(&15.0));
        assert_eq!(stats.get("count"), Some(&5.0));
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;

pub struct DataProcessor {
    records: Vec<HashMap<String, f64>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        if let Some(header_result) = lines.next() {
            let header_line = header_result?;
            let headers: Vec<String> = header_line.split(',').map(|s| s.trim().to_string()).collect();

            for line_result in lines {
                let line = line_result?;
                let values: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
                
                if values.len() == headers.len() {
                    let mut record = HashMap::new();
                    for (i, header) in headers.iter().enumerate() {
                        if let Ok(num) = values[i].parse::<f64>() {
                            record.insert(header.clone(), num);
                        }
                    }
                    if !record.is_empty() {
                        self.records.push(record);
                    }
                }
            }
        }
        
        Ok(())
    }

    pub fn calculate_statistics(&self, column_name: &str) -> Option<(f64, f64, f64)> {
        let values: Vec<f64> = self.records
            .iter()
            .filter_map(|record| record.get(column_name).copied())
            .collect();

        if values.is_empty() {
            return None;
        }

        let sum: f64 = values.iter().sum();
        let count = values.len() as f64;
        let mean = sum / count;

        let variance: f64 = values.iter()
            .map(|value| (value - mean).powi(2))
            .sum::<f64>() / count;

        let std_dev = variance.sqrt();

        Some((mean, variance, std_dev))
    }

    pub fn filter_records(&self, column_name: &str, threshold: f64) -> Vec<HashMap<String, f64>> {
        self.records
            .iter()
            .filter(|record| {
                record.get(column_name)
                    .map(|&value| value > threshold)
                    .unwrap_or(false)
            })
            .cloned()
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
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,score").unwrap();
        writeln!(temp_file, "1,10.5,0.8").unwrap();
        writeln!(temp_file, "2,15.2,0.9").unwrap();
        writeln!(temp_file, "3,8.7,0.6").unwrap();
        
        let result = processor.load_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(processor.record_count(), 3);
        
        let stats = processor.calculate_statistics("value");
        assert!(stats.is_some());
        
        let (mean, _, std_dev) = stats.unwrap();
        assert!((mean - 11.466666).abs() < 0.001);
        assert!((std_dev - 2.749545).abs() < 0.001);
        
        let filtered = processor.filter_records("value", 10.0);
        assert_eq!(filtered.len(), 2);
    }
}
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    id: u32,
    values: Vec<f64>,
    metadata: HashMap<String, String>,
}

#[derive(Debug)]
pub enum ProcessingError {
    InvalidData(String),
    TransformationError(String),
    ValidationError(String),
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessingError::InvalidData(msg) => write!(f, "Invalid data: {}", msg),
            ProcessingError::TransformationError(msg) => write!(f, "Transformation error: {}", msg),
            ProcessingError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl Error for ProcessingError {}

impl DataRecord {
    pub fn new(id: u32, values: Vec<f64>) -> Result<Self, ProcessingError> {
        if values.is_empty() {
            return Err(ProcessingError::InvalidData("Values cannot be empty".to_string()));
        }
        
        if values.iter().any(|&v| v.is_nan() || v.is_infinite()) {
            return Err(ProcessingError::InvalidData("Values contain NaN or infinite numbers".to_string()));
        }
        
        Ok(Self {
            id,
            values,
            metadata: HashMap::new(),
        })
    }
    
    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }
    
    pub fn validate(&self) -> Result<(), ProcessingError> {
        if self.id == 0 {
            return Err(ProcessingError::ValidationError("ID cannot be zero".to_string()));
        }
        
        if self.values.len() > 1000 {
            return Err(ProcessingError::ValidationError("Too many values".to_string()));
        }
        
        Ok(())
    }
    
    pub fn normalize(&mut self) -> Result<(), ProcessingError> {
        let min = self.values
            .iter()
            .fold(f64::INFINITY, |a, &b| a.min(b));
        let max = self.values
            .iter()
            .fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        
        if max - min < f64::EPSILON {
            return Err(ProcessingError::TransformationError(
                "Cannot normalize: all values are equal".to_string()
            ));
        }
        
        for value in &mut self.values {
            *value = (*value - min) / (max - min);
        }
        
        Ok(())
    }
    
    pub fn calculate_statistics(&self) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        
        let sum: f64 = self.values.iter().sum();
        let count = self.values.len() as f64;
        let mean = sum / count;
        
        let variance: f64 = self.values
            .iter()
            .map(|&v| (v - mean).powi(2))
            .sum::<f64>() / count;
        
        stats.insert("mean".to_string(), mean);
        stats.insert("variance".to_string(), variance);
        stats.insert("min".to_string(), self.values.iter().fold(f64::INFINITY, |a, &b| a.min(b)));
        stats.insert("max".to_string(), self.values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b)));
        stats.insert("sum".to_string(), sum);
        stats.insert("count".to_string(), count);
        
        stats
    }
}

pub fn process_records(records: &mut [DataRecord]) -> Result<Vec<HashMap<String, f64>>, ProcessingError> {
    let mut results = Vec::new();
    
    for record in records {
        record.validate()?;
        record.normalize()?;
        results.push(record.calculate_statistics());
    }
    
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_valid_record_creation() {
        let record = DataRecord::new(1, vec![1.0, 2.0, 3.0]);
        assert!(record.is_ok());
    }
    
    #[test]
    fn test_invalid_record_empty_values() {
        let record = DataRecord::new(1, vec![]);
        assert!(record.is_err());
    }
    
    #[test]
    fn test_normalization() {
        let mut record = DataRecord::new(1, vec![1.0, 2.0, 3.0]).unwrap();
        assert!(record.normalize().is_ok());
        
        let values = record.values;
        assert!((values[0] - 0.0).abs() < f64::EPSILON);
        assert!((values[1] - 0.5).abs() < f64::EPSILON);
        assert!((values[2] - 1.0).abs() < f64::EPSILON);
    }
    
    #[test]
    fn test_statistics_calculation() {
        let record = DataRecord::new(1, vec![1.0, 2.0, 3.0, 4.0]).unwrap();
        let stats = record.calculate_statistics();
        
        assert!((stats["mean"] - 2.5).abs() < f64::EPSILON);
        assert!((stats["variance"] - 1.25).abs() < f64::EPSILON);
        assert!((stats["min"] - 1.0).abs() < f64::EPSILON);
        assert!((stats["max"] - 4.0).abs() < f64::EPSILON);
        assert!((stats["sum"] - 10.0).abs() < f64::EPSILON);
        assert!((stats["count"] - 4.0).abs() < f64::EPSILON);
    }
}use csv::Reader;
use serde::Deserialize;
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize)]
struct Record {
    id: u32,
    value: f64,
    category: String,
}

pub struct DataProcessor {
    records: Vec<Record>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let mut rdr = Reader::from_reader(file);

        for result in rdr.deserialize() {
            let record: Record = result?;
            self.records.push(record);
        }

        Ok(())
    }

    pub fn calculate_mean(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }

        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|r| r.category == category)
            .collect()
    }

    pub fn max_value(&self) -> Option<f64> {
        self.records.iter().map(|r| r.value).reduce(f64::max)
    }

    pub fn min_value(&self) -> Option<f64> {
        self.records.iter().map(|r| r.value).reduce(f64::min)
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
    fn test_data_processing() {
        let mut processor = DataProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category").unwrap();
        writeln!(temp_file, "1,10.5,A").unwrap();
        writeln!(temp_file, "2,20.3,B").unwrap();
        writeln!(temp_file, "3,15.7,A").unwrap();
        
        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(processor.record_count(), 3);
        
        let mean = processor.calculate_mean().unwrap();
        assert!((mean - 15.5).abs() < 0.01);
        
        let category_a = processor.filter_by_category("A");
        assert_eq!(category_a.len(), 2);
        
        let max_val = processor.max_value().unwrap();
        assert!((max_val - 20.3).abs() < 0.01);
    }
}