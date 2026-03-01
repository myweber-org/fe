
use std::error::Error;
use std::fs::File;
use std::path::Path;

pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub category: String,
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<usize, Box<dyn Error>> {
        let path = Path::new(file_path);
        let file = File::open(path)?;
        let mut rdr = csv::Reader::from_reader(file);
        
        let mut count = 0;
        for result in rdr.deserialize() {
            let record: DataRecord = result?;
            self.validate_record(&record)?;
            self.records.push(record);
            count += 1;
        }
        
        Ok(count)
    }

    fn validate_record(&self, record: &DataRecord) -> Result<(), Box<dyn Error>> {
        if record.value < 0.0 {
            return Err("Negative values are not allowed".into());
        }
        if record.category.is_empty() {
            return Err("Category cannot be empty".into());
        }
        Ok(())
    }

    pub fn calculate_average(&self) -> Option<f64> {
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

    pub fn get_statistics(&self) -> Statistics {
        let values: Vec<f64> = self.records.iter().map(|r| r.value).collect();
        
        Statistics {
            count: self.records.len(),
            min: values.iter().copied().fold(f64::INFINITY, f64::min),
            max: values.iter().copied().fold(f64::NEG_INFINITY, f64::max),
            average: self.calculate_average().unwrap_or(0.0),
        }
    }
}

pub struct Statistics {
    pub count: usize,
    pub min: f64,
    pub max: f64,
    pub average: f64,
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
        writeln!(temp_file, "1,10.5,alpha").unwrap();
        writeln!(temp_file, "2,20.3,beta").unwrap();
        writeln!(temp_file, "3,15.7,alpha").unwrap();
        
        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 3);
        
        let stats = processor.get_statistics();
        assert_eq!(stats.count, 3);
        assert_eq!(stats.min, 10.5);
        assert_eq!(stats.max, 20.3);
        
        let alpha_records = processor.filter_by_category("alpha");
        assert_eq!(alpha_records.len(), 2);
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct DataProcessor {
    delimiter: char,
    has_header: bool,
}

impl DataProcessor {
    pub fn new(delimiter: char, has_header: bool) -> Self {
        DataProcessor {
            delimiter,
            has_header,
        }
    }

    pub fn process_file<P: AsRef<Path>>(&self, file_path: P) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();
        let mut lines = reader.lines();

        if self.has_header {
            lines.next();
        }

        for line_result in lines {
            let line = line_result?;
            let fields: Vec<String> = line.split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();
            
            if !fields.is_empty() {
                records.push(fields);
            }
        }

        Ok(records)
    }

    pub fn validate_record(&self, record: &[String], expected_fields: usize) -> bool {
        record.len() == expected_fields && record.iter().all(|field| !field.is_empty())
    }

    pub fn extract_column(&self, data: &[Vec<String>], column_index: usize) -> Vec<String> {
        data.iter()
            .filter_map(|record| record.get(column_index))
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
    fn test_process_csv() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "John,30,New York").unwrap();
        writeln!(temp_file, "Alice,25,London").unwrap();

        let processor = DataProcessor::new(',', true);
        let result = processor.process_file(temp_file.path()).unwrap();
        
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["John", "30", "New York"]);
    }

    #[test]
    fn test_validate_record() {
        let processor = DataProcessor::new(',', false);
        let valid_record = vec!["data1".to_string(), "data2".to_string()];
        let invalid_record = vec!["".to_string(), "data2".to_string()];
        
        assert!(processor.validate_record(&valid_record, 2));
        assert!(!processor.validate_record(&invalid_record, 2));
    }

    #[test]
    fn test_extract_column() {
        let processor = DataProcessor::new(',', false);
        let data = vec![
            vec!["a".to_string(), "b".to_string()],
            vec!["c".to_string(), "d".to_string()],
        ];
        
        let column = processor.extract_column(&data, 1);
        assert_eq!(column, vec!["b".to_string(), "d".to_string()]);
    }
}
use std::collections::HashMap;

pub struct DataProcessor {
    cache: HashMap<String, Vec<f64>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            cache: HashMap::new(),
        }
    }

    pub fn process_numeric_data(&mut self, key: &str, values: &[f64]) -> Result<Vec<f64>, String> {
        if values.is_empty() {
            return Err("Empty data provided".to_string());
        }

        if let Some(cached) = self.cache.get(key) {
            return Ok(cached.clone());
        }

        let processed: Vec<f64> = values
            .iter()
            .filter(|&&x| x.is_finite())
            .map(|&x| x * 2.0)
            .collect();

        if processed.len() < values.len() / 2 {
            return Err("Too many invalid values".to_string());
        }

        self.cache.insert(key.to_string(), processed.clone());
        Ok(processed)
    }

    pub fn calculate_statistics(&self, data: &[f64]) -> (f64, f64, f64) {
        if data.is_empty() {
            return (0.0, 0.0, 0.0);
        }

        let sum: f64 = data.iter().sum();
        let mean = sum / data.len() as f64;
        
        let variance: f64 = data.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / data.len() as f64;
        
        let std_dev = variance.sqrt();
        
        (mean, variance, std_dev)
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

    #[test]
    fn test_process_valid_data() {
        let mut processor = DataProcessor::new();
        let data = vec![1.0, 2.0, 3.0];
        let result = processor.process_numeric_data("test", &data);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec![2.0, 4.0, 6.0]);
    }

    #[test]
    fn test_cache_functionality() {
        let mut processor = DataProcessor::new();
        let data = vec![1.0, 2.0];
        
        let first_result = processor.process_numeric_data("cached", &data);
        assert!(first_result.is_ok());
        
        let second_result = processor.process_numeric_data("cached", &data);
        assert!(second_result.is_ok());
        
        assert_eq!(processor.cache_size(), 1);
    }

    #[test]
    fn test_statistics_calculation() {
        let processor = DataProcessor::new();
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let (mean, variance, std_dev) = processor.calculate_statistics(&data);
        
        assert!((mean - 3.0).abs() < 0.0001);
        assert!((variance - 2.0).abs() < 0.0001);
        assert!((std_dev - 1.4142).abs() < 0.0001);
    }
}
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    id: u32,
    name: String,
    value: f64,
    tags: Vec<String>,
    metadata: HashMap<String, String>,
}

#[derive(Debug)]
pub enum ValidationError {
    InvalidId,
    EmptyName,
    NegativeValue,
    MissingRequiredTag,
    DuplicateMetadataKey,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::InvalidId => write!(f, "ID must be greater than zero"),
            ValidationError::EmptyName => write!(f, "Name cannot be empty"),
            ValidationError::NegativeValue => write!(f, "Value must be non-negative"),
            ValidationError::MissingRequiredTag => write!(f, "Required tag is missing"),
            ValidationError::DuplicateMetadataKey => write!(f, "Duplicate metadata key found"),
        }
    }
}

impl Error for ValidationError {}

impl DataRecord {
    pub fn new(id: u32, name: String, value: f64) -> Self {
        DataRecord {
            id,
            name,
            value,
            tags: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.id == 0 {
            return Err(ValidationError::InvalidId);
        }
        
        if self.name.trim().is_empty() {
            return Err(ValidationError::EmptyName);
        }
        
        if self.value < 0.0 {
            return Err(ValidationError::NegativeValue);
        }
        
        if !self.tags.contains(&"verified".to_string()) {
            return Err(ValidationError::MissingRequiredTag);
        }
        
        Ok(())
    }

    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
    }

    pub fn add_metadata(&mut self, key: String, value: String) -> Result<(), ValidationError> {
        if self.metadata.contains_key(&key) {
            return Err(ValidationError::DuplicateMetadataKey);
        }
        self.metadata.insert(key, value);
        Ok(())
    }

    pub fn transform_value(&mut self, multiplier: f64) {
        if multiplier > 0.0 {
            self.value *= multiplier;
        }
    }

    pub fn calculate_score(&self) -> f64 {
        let base_score = self.value * 100.0;
        let tag_bonus = self.tags.len() as f64 * 10.0;
        let metadata_bonus = self.metadata.len() as f64 * 5.0;
        
        base_score + tag_bonus + metadata_bonus
    }

    pub fn to_json_string(&self) -> String {
        let tags_json: Vec<String> = self.tags.iter()
            .map(|tag| format!("\"{}\"", tag))
            .collect();
        
        let metadata_json: Vec<String> = self.metadata.iter()
            .map(|(k, v)| format!("\"{}\": \"{}\"", k, v))
            .collect();
        
        format!(
            "{{\"id\": {}, \"name\": \"{}\", \"value\": {}, \"tags\": [{}], \"metadata\": {{{}}}}}",
            self.id,
            self.name,
            self.value,
            tags_json.join(", "),
            metadata_json.join(", ")
        )
    }
}

pub fn process_records(records: &mut [DataRecord]) -> Vec<Result<f64, ValidationError>> {
    records.iter_mut()
        .map(|record| {
            record.validate()?;
            record.add_tag("processed".to_string());
            record.transform_value(1.1);
            Ok(record.calculate_score())
        })
        .collect()
}

pub fn filter_valid_records(records: &[DataRecord]) -> Vec<&DataRecord> {
    records.iter()
        .filter(|record| record.validate().is_ok())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_creation() {
        let record = DataRecord::new(1, "Test Record".to_string(), 42.5);
        assert_eq!(record.id, 1);
        assert_eq!(record.name, "Test Record");
        assert_eq!(record.value, 42.5);
    }

    #[test]
    fn test_validation_success() {
        let mut record = DataRecord::new(1, "Valid Record".to_string(), 100.0);
        record.add_tag("verified".to_string());
        assert!(record.validate().is_ok());
    }

    #[test]
    fn test_validation_failure() {
        let record = DataRecord::new(0, "".to_string(), -10.0);
        assert!(record.validate().is_err());
    }

    #[test]
    fn test_score_calculation() {
        let mut record = DataRecord::new(1, "Test".to_string(), 50.0);
        record.add_tag("tag1".to_string());
        record.add_tag("tag2".to_string());
        record.add_metadata("key1".to_string(), "value1".to_string()).unwrap();
        
        let score = record.calculate_score();
        assert_eq!(score, 50.0 * 100.0 + 2.0 * 10.0 + 1.0 * 5.0);
    }
}
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

pub fn process_csv_file(file_path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let mut reader = Reader::from_reader(file);
    let mut records = Vec::new();

    for result in reader.deserialize() {
        let record: Record = result?;
        if record.value >= 0.0 {
            records.push(record);
        }
    }

    Ok(records)
}

pub fn calculate_statistics(records: &[Record]) -> (f64, f64, f64) {
    if records.is_empty() {
        return (0.0, 0.0, 0.0);
    }

    let sum: f64 = records.iter().map(|r| r.value).sum();
    let count = records.len() as f64;
    let mean = sum / count;

    let variance: f64 = records
        .iter()
        .map(|r| (r.value - mean).powi(2))
        .sum::<f64>() / count;

    let std_dev = variance.sqrt();

    (mean, variance, std_dev)
}

pub fn filter_by_category(records: Vec<Record>, category: &str) -> Vec<Record> {
    records
        .into_iter()
        .filter(|r| r.category == category)
        .collect()
}