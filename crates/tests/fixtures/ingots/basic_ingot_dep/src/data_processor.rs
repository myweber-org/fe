use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct DataProcessor {
    file_path: String,
}

impl DataProcessor {
    pub fn new(file_path: &str) -> Self {
        DataProcessor {
            file_path: file_path.to_string(),
        }
    }

    pub fn process_csv(&self, filter_column: usize, filter_value: &str) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let path = Path::new(&self.file_path);
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        
        let mut filtered_data = Vec::new();
        
        for line in reader.lines() {
            let line = line?;
            let columns: Vec<String> = line.split(',').map(|s| s.trim().to_string()).collect();
            
            if columns.len() > filter_column && columns[filter_column] == filter_value {
                filtered_data.push(columns);
            }
        }
        
        Ok(filtered_data)
    }

    pub fn calculate_average(&self, column_index: usize) -> Result<f64, Box<dyn Error>> {
        let path = Path::new(&self.file_path);
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        
        let mut sum = 0.0;
        let mut count = 0;
        
        for line in reader.lines().skip(1) {
            let line = line?;
            let columns: Vec<&str> = line.split(',').collect();
            
            if let Some(value_str) = columns.get(column_index) {
                if let Ok(value) = value_str.trim().parse::<f64>() {
                    sum += value;
                    count += 1;
                }
            }
        }
        
        if count > 0 {
            Ok(sum / count as f64)
        } else {
            Ok(0.0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_process_csv() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,age,department").unwrap();
        writeln!(temp_file, "1,John,30,Engineering").unwrap();
        writeln!(temp_file, "2,Jane,25,Marketing").unwrap();
        writeln!(temp_file, "3,Bob,35,Engineering").unwrap();
        
        let processor = DataProcessor::new(temp_file.path().to_str().unwrap());
        let result = processor.process_csv(3, "Engineering").unwrap();
        
        assert_eq!(result.len(), 2);
        assert_eq!(result[0][1], "John");
        assert_eq!(result[1][1], "Bob");
    }

    #[test]
    fn test_calculate_average() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,age").unwrap();
        writeln!(temp_file, "1,John,30").unwrap();
        writeln!(temp_file, "2,Jane,25").unwrap();
        writeln!(temp_file, "3,Bob,35").unwrap();
        
        let processor = DataProcessor::new(temp_file.path().to_str().unwrap());
        let average = processor.calculate_average(2).unwrap();
        
        assert!((average - 30.0).abs() < 0.001);
    }
}
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub category: String,
}

#[derive(Debug)]
pub enum ValidationError {
    InvalidId,
    InvalidValue,
    EmptyCategory,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::InvalidId => write!(f, "ID must be greater than 0"),
            ValidationError::InvalidValue => write!(f, "Value must be between 0.0 and 1000.0"),
            ValidationError::EmptyCategory => write!(f, "Category cannot be empty"),
        }
    }
}

impl Error for ValidationError {}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: String) -> Result<Self, ValidationError> {
        if id == 0 {
            return Err(ValidationError::InvalidId);
        }
        
        if value < 0.0 || value > 1000.0 {
            return Err(ValidationError::InvalidValue);
        }
        
        if category.trim().is_empty() {
            return Err(ValidationError::EmptyCategory);
        }
        
        Ok(Self {
            id,
            value,
            category: category.trim().to_string(),
        })
    }
    
    pub fn normalize_value(&self) -> f64 {
        self.value / 1000.0
    }
    
    pub fn to_uppercase_category(&self) -> String {
        self.category.to_uppercase()
    }
}

pub fn process_records(records: &[DataRecord]) -> Vec<(u32, f64, String)> {
    records
        .iter()
        .map(|record| {
            (
                record.id,
                record.normalize_value(),
                record.to_uppercase_category(),
            )
        })
        .collect()
}

pub fn filter_by_threshold(records: &[DataRecord], threshold: f64) -> Vec<&DataRecord> {
    records
        .iter()
        .filter(|record| record.value >= threshold)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_valid_record_creation() {
        let record = DataRecord::new(1, 500.0, "Test".to_string());
        assert!(record.is_ok());
        
        let record = record.unwrap();
        assert_eq!(record.id, 1);
        assert_eq!(record.value, 500.0);
        assert_eq!(record.category, "Test");
    }
    
    #[test]
    fn test_invalid_id() {
        let record = DataRecord::new(0, 500.0, "Test".to_string());
        assert!(matches!(record, Err(ValidationError::InvalidId)));
    }
    
    #[test]
    fn test_normalize_value() {
        let record = DataRecord::new(1, 500.0, "Test".to_string()).unwrap();
        assert_eq!(record.normalize_value(), 0.5);
    }
    
    #[test]
    fn test_process_records() {
        let records = vec![
            DataRecord::new(1, 200.0, "alpha".to_string()).unwrap(),
            DataRecord::new(2, 800.0, "beta".to_string()).unwrap(),
        ];
        
        let processed = process_records(&records);
        assert_eq!(processed.len(), 2);
        assert_eq!(processed[0].1, 0.2);
        assert_eq!(processed[1].2, "BETA");
    }
}
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub tags: Vec<String>,
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
    config: ProcessorConfig,
    cache: HashMap<u32, DataRecord>,
}

#[derive(Debug, Clone)]
pub struct ProcessorConfig {
    pub max_value: f64,
    pub min_value: f64,
    pub allowed_tags: Vec<String>,
}

impl DataProcessor {
    pub fn new(config: ProcessorConfig) -> Self {
        DataProcessor {
            config,
            cache: HashMap::new(),
        }
    }

    pub fn process_record(&mut self, record: DataRecord) -> Result<DataRecord, ProcessingError> {
        self.validate_record(&record)?;
        
        let transformed = self.transform_record(record)?;
        
        self.cache.insert(transformed.id, transformed.clone());
        
        Ok(transformed)
    }

    fn validate_record(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if record.name.is_empty() {
            return Err(ProcessingError::ValidationError(
                "Record name cannot be empty".to_string(),
            ));
        }

        if record.value < self.config.min_value || record.value > self.config.max_value {
            return Err(ProcessingError::ValidationError(format!(
                "Value {} is outside allowed range [{}, {}]",
                record.value, self.config.min_value, self.config.max_value
            )));
        }

        for tag in &record.tags {
            if !self.config.allowed_tags.contains(tag) {
                return Err(ProcessingError::ValidationError(format!(
                    "Tag '{}' is not in allowed tags list",
                    tag
                )));
            }
        }

        Ok(())
    }

    fn transform_record(&self, mut record: DataRecord) -> Result<DataRecord, ProcessingError> {
        record.name = record.name.trim().to_string();
        
        if record.name.is_empty() {
            return Err(ProcessingError::TransformationFailed(
                "Name became empty after trimming".to_string(),
            ));
        }

        record.value = (record.value * 100.0).round() / 100.0;

        record.tags.sort();
        record.tags.dedup();

        Ok(record)
    }

    pub fn get_cached_record(&self, id: u32) -> Option<&DataRecord> {
        self.cache.get(&id)
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    pub fn cache_size(&self) -> usize {
        self.cache.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> ProcessorConfig {
        ProcessorConfig {
            max_value: 1000.0,
            min_value: 0.0,
            allowed_tags: vec!["alpha".to_string(), "beta".to_string(), "gamma".to_string()],
        }
    }

    #[test]
    fn test_valid_record_processing() {
        let config = create_test_config();
        let mut processor = DataProcessor::new(config);

        let record = DataRecord {
            id: 1,
            name: "  Test Record  ".to_string(),
            value: 123.456,
            tags: vec!["alpha".to_string(), "beta".to_string()],
        };

        let result = processor.process_record(record);
        assert!(result.is_ok());

        let processed = result.unwrap();
        assert_eq!(processed.name, "Test Record");
        assert_eq!(processed.value, 123.46);
        assert_eq!(processed.tags, vec!["alpha".to_string(), "beta".to_string()]);
        assert_eq!(processor.cache_size(), 1);
    }

    #[test]
    fn test_invalid_value() {
        let config = create_test_config();
        let mut processor = DataProcessor::new(config);

        let record = DataRecord {
            id: 2,
            name: "Invalid Value".to_string(),
            value: 1500.0,
            tags: vec!["alpha".to_string()],
        };

        let result = processor.process_record(record);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_tag() {
        let config = create_test_config();
        let mut processor = DataProcessor::new(config);

        let record = DataRecord {
            id: 3,
            name: "Invalid Tag".to_string(),
            value: 500.0,
            tags: vec!["delta".to_string()],
        };

        let result = processor.process_record(record);
        assert!(result.is_err());
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;

pub struct DataProcessor {
    data: Vec<f64>,
    metadata: HashMap<String, String>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            data: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn load_from_csv(&mut self, filepath: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(filepath)?;
        let reader = BufReader::new(file);
        
        self.data.clear();
        
        for (index, line) in reader.lines().enumerate() {
            let line = line?;
            if index == 0 {
                continue;
            }
            
            if let Ok(value) = line.trim().parse::<f64>() {
                self.data.push(value);
            }
        }
        
        self.metadata.insert("source".to_string(), filepath.to_string());
        self.metadata.insert("loaded_at".to_string(), chrono::Local::now().to_rfc3339());
        
        Ok(())
    }

    pub fn calculate_statistics(&self) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        
        if self.data.is_empty() {
            return stats;
        }
        
        let sum: f64 = self.data.iter().sum();
        let count = self.data.len() as f64;
        let mean = sum / count;
        
        let variance: f64 = self.data.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / count;
        
        let std_dev = variance.sqrt();
        
        let min = self.data.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = self.data.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        
        stats.insert("mean".to_string(), mean);
        stats.insert("std_dev".to_string(), std_dev);
        stats.insert("min".to_string(), min);
        stats.insert("max".to_string(), max);
        stats.insert("count".to_string(), count);
        stats.insert("sum".to_string(), sum);
        
        stats
    }

    pub fn filter_data(&self, threshold: f64) -> Vec<f64> {
        self.data.iter()
            .filter(|&&x| x >= threshold)
            .cloned()
            .collect()
    }

    pub fn get_metadata(&self) -> &HashMap<String, String> {
        &self.metadata
    }

    pub fn data_count(&self) -> usize {
        self.data.len()
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
        writeln!(temp_file, "value\n10.5\n20.3\n15.7\n25.1\n18.9").unwrap();
        
        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(processor.data_count(), 5);
        
        let stats = processor.calculate_statistics();
        assert_eq!(stats.get("count").unwrap(), &5.0);
        
        let filtered = processor.filter_data(15.0);
        assert_eq!(filtered.len(), 4);
        
        let metadata = processor.get_metadata();
        assert!(metadata.contains_key("source"));
    }
}