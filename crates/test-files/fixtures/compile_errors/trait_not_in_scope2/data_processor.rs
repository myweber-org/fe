
use std::collections::HashMap;
use std::error::Error;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

impl DataRecord {
    pub fn new(id: u32, values: Vec<f64>) -> Self {
        Self {
            id,
            values,
            metadata: HashMap::new(),
        }
    }

    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    pub fn validate(&self) -> Result<(), Box<dyn Error>> {
        if self.id == 0 {
            return Err("Invalid record ID".into());
        }
        
        if self.values.is_empty() {
            return Err("Empty values vector".into());
        }

        for value in &self.values {
            if value.is_nan() || value.is_infinite() {
                return Err("Invalid numeric value detected".into());
            }
        }

        Ok(())
    }

    pub fn normalize(&mut self) {
        if let Some(max) = self.values.iter().copied().reduce(f64::max) {
            if max != 0.0 {
                for value in &mut self.values {
                    *value /= max;
                }
            }
        }
    }

    pub fn calculate_statistics(&self) -> (f64, f64, f64) {
        let count = self.values.len() as f64;
        let sum: f64 = self.values.iter().sum();
        let mean = sum / count;

        let variance: f64 = self.values
            .iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / count;

        let std_dev = variance.sqrt();

        (mean, variance, std_dev)
    }
}

pub fn process_records(records: &mut [DataRecord]) -> Result<Vec<DataRecord>, Box<dyn Error>> {
    let mut processed = Vec::new();

    for record in records {
        record.validate()?;
        let mut processed_record = record.clone();
        processed_record.normalize();
        processed.push(processed_record);
    }

    Ok(processed)
}

pub fn aggregate_statistics(records: &[DataRecord]) -> HashMap<String, f64> {
    let mut stats = HashMap::new();
    let mut total_mean = 0.0;
    let mut total_variance = 0.0;
    let mut count = 0;

    for record in records {
        let (mean, variance, std_dev) = record.calculate_statistics();
        total_mean += mean;
        total_variance += variance;
        count += 1;

        stats.insert(format!("record_{}_mean", record.id), mean);
        stats.insert(format!("record_{}_std_dev", record.id), std_dev);
    }

    if count > 0 {
        stats.insert("overall_mean".to_string(), total_mean / count as f64);
        stats.insert("overall_variance".to_string(), total_variance / count as f64);
    }

    stats
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_validation() {
        let valid_record = DataRecord::new(1, vec![1.0, 2.0, 3.0]);
        assert!(valid_record.validate().is_ok());

        let invalid_record = DataRecord::new(0, vec![1.0, 2.0]);
        assert!(invalid_record.validate().is_err());

        let nan_record = DataRecord::new(2, vec![1.0, f64::NAN]);
        assert!(nan_record.validate().is_err());
    }

    #[test]
    fn test_normalization() {
        let mut record = DataRecord::new(1, vec![1.0, 2.0, 3.0]);
        record.normalize();
        assert_eq!(record.values, vec![1.0/3.0, 2.0/3.0, 1.0]);
    }

    #[test]
    fn test_statistics_calculation() {
        let record = DataRecord::new(1, vec![1.0, 2.0, 3.0]);
        let (mean, variance, std_dev) = record.calculate_statistics();
        
        assert!((mean - 2.0).abs() < 1e-10);
        assert!((variance - 2.0/3.0).abs() < 1e-10);
        assert!((std_dev - (2.0/3.0 as f64).sqrt()).abs() < 1e-10);
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

pub struct DataProcessor {
    config: ProcessingConfig,
}

#[derive(Debug, Clone)]
pub struct ProcessingConfig {
    pub max_values: usize,
    pub min_timestamp: i64,
    pub max_timestamp: i64,
    pub allowed_metadata_keys: Vec<String>,
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

        if record.timestamp < self.config.min_timestamp
            || record.timestamp > self.config.max_timestamp
        {
            return Err(ProcessingError::ValidationError(format!(
                "Timestamp {} out of range [{}, {}]",
                record.timestamp, self.config.min_timestamp, self.config.max_timestamp
            )));
        }

        for key in record.metadata.keys() {
            if !self.config.allowed_metadata_keys.contains(key) {
                return Err(ProcessingError::ValidationError(format!(
                    "Metadata key '{}' not allowed",
                    key
                )));
            }
        }

        Ok(())
    }

    pub fn transform_record(
        &self,
        record: &DataRecord,
    ) -> Result<TransformedRecord, ProcessingError> {
        self.validate_record(record)?;

        let sum: f64 = record.values.iter().sum();
        let avg = if !record.values.is_empty() {
            sum / record.values.len() as f64
        } else {
            0.0
        };

        let variance: f64 = if record.values.len() > 1 {
            let mean = avg;
            record
                .values
                .iter()
                .map(|&x| (x - mean).powi(2))
                .sum::<f64>()
                / (record.values.len() - 1) as f64
        } else {
            0.0
        };

        let metadata_count = record.metadata.len();

        Ok(TransformedRecord {
            original_id: record.id,
            timestamp: record.timestamp,
            value_count: record.values.len(),
            value_sum: sum,
            value_average: avg,
            value_variance: variance,
            metadata_count,
            processed_at: chrono::Utc::now().timestamp(),
        })
    }

    pub fn process_batch(
        &self,
        records: Vec<DataRecord>,
    ) -> Result<Vec<TransformedRecord>, ProcessingError> {
        let mut results = Vec::with_capacity(records.len());
        let mut errors = Vec::new();

        for (index, record) in records.into_iter().enumerate() {
            match self.transform_record(&record) {
                Ok(transformed) => results.push(transformed),
                Err(e) => errors.push((index, e)),
            }
        }

        if !errors.is_empty() {
            let error_msg = errors
                .iter()
                .map(|(idx, err)| format!("Record {}: {}", idx, err))
                .collect::<Vec<String>>()
                .join("; ");
            return Err(ProcessingError::ProcessingError(format!(
                "Batch processing failed with errors: {}",
                error_msg
            )));
        }

        Ok(results)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformedRecord {
    pub original_id: u64,
    pub timestamp: i64,
    pub value_count: usize,
    pub value_sum: f64,
    pub value_average: f64,
    pub value_variance: f64,
    pub metadata_count: usize,
    pub processed_at: i64,
}

impl ProcessingError {
    fn ProcessingError(msg: String) -> Self {
        ProcessingError::TransformationError(msg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> ProcessingConfig {
        ProcessingConfig {
            max_values: 10,
            min_timestamp: 0,
            max_timestamp: 1000000000,
            allowed_metadata_keys: vec!["source".to_string(), "type".to_string()],
        }
    }

    fn create_valid_record() -> DataRecord {
        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), "test".to_string());

        DataRecord {
            id: 1,
            timestamp: 123456789,
            values: vec![1.0, 2.0, 3.0, 4.0, 5.0],
            metadata,
        }
    }

    #[test]
    fn test_valid_record_validation() {
        let processor = DataProcessor::new(create_test_config());
        let record = create_valid_record();
        assert!(processor.validate_record(&record).is_ok());
    }

    #[test]
    fn test_invalid_timestamp() {
        let processor = DataProcessor::new(create_test_config());
        let mut record = create_valid_record();
        record.timestamp = -1;
        assert!(processor.validate_record(&record).is_err());
    }

    #[test]
    fn test_too_many_values() {
        let processor = DataProcessor::new(create_test_config());
        let mut record = create_valid_record();
        record.values = vec![1.0; 11];
        assert!(processor.validate_record(&record).is_err());
    }

    #[test]
    fn test_invalid_metadata_key() {
        let processor = DataProcessor::new(create_test_config());
        let mut record = create_valid_record();
        record.metadata.insert("invalid".to_string(), "value".to_string());
        assert!(processor.validate_record(&record).is_err());
    }

    #[test]
    fn test_record_transformation() {
        let processor = DataProcessor::new(create_test_config());
        let record = create_valid_record();
        let transformed = processor.transform_record(&record).unwrap();

        assert_eq!(transformed.original_id, 1);
        assert_eq!(transformed.timestamp, 123456789);
        assert_eq!(transformed.value_count, 5);
        assert_eq!(transformed.value_sum, 15.0);
        assert_eq!(transformed.value_average, 3.0);
        assert_eq!(transformed.metadata_count, 1);
    }

    #[test]
    fn test_batch_processing() {
        let processor = DataProcessor::new(create_test_config());
        let records = vec![create_valid_record(), create_valid_record()];
        let results = processor.process_batch(records).unwrap();

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].original_id, 1);
        assert_eq!(results[1].original_id, 1);
    }

    #[test]
    fn test_batch_processing_with_error() {
        let processor = DataProcessor::new(create_test_config());
        let mut invalid_record = create_valid_record();
        invalid_record.timestamp = -1;

        let records = vec![create_valid_record(), invalid_record];
        let result = processor.process_batch(records);

        assert!(result.is_err());
    }
}
use std::error::Error;
use std::fs::File;
use std::path::Path;

pub struct DataProcessor {
    data: Vec<Vec<String>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor { data: Vec::new() }
    }

    pub fn load_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let path = Path::new(file_path);
        let file = File::open(path)?;
        let mut rdr = csv::Reader::from_reader(file);
        
        for result in rdr.records() {
            let record = result?;
            let row: Vec<String> = record.iter().map(|field| field.to_string()).collect();
            self.data.push(row);
        }
        
        Ok(())
    }

    pub fn validate_data(&self) -> bool {
        if self.data.is_empty() {
            return false;
        }
        
        let header_len = self.data[0].len();
        for row in &self.data[1..] {
            if row.len() != header_len {
                return false;
            }
        }
        
        true
    }

    pub fn get_row_count(&self) -> usize {
        self.data.len()
    }

    pub fn get_column_count(&self) -> usize {
        if self.data.is_empty() {
            0
        } else {
            self.data[0].len()
        }
    }

    pub fn filter_rows<F>(&self, predicate: F) -> Vec<Vec<String>>
    where
        F: Fn(&[String]) -> bool,
    {
        self.data.iter()
            .filter(|row| predicate(row))
            .cloned()
            .collect()
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
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,30,New York").unwrap();
        writeln!(temp_file, "Bob,25,London").unwrap();
        
        let result = processor.load_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(processor.get_row_count(), 3);
        assert_eq!(processor.get_column_count(), 3);
        assert!(processor.validate_data());
        
        let filtered = processor.filter_rows(|row| {
            row.get(1).and_then(|age| age.parse::<i32>().ok()).map_or(false, |age| age > 25)
        });
        
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0][0], "Alice");
    }
}