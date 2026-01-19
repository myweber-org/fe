
use std::collections::HashMap;

pub struct DataProcessor {
    cache: HashMap<String, Vec<f64>>,
    validation_rules: Vec<ValidationRule>,
}

pub struct ValidationRule {
    field_name: String,
    min_value: f64,
    max_value: f64,
    required: bool,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            cache: HashMap::new(),
            validation_rules: Vec::new(),
        }
    }

    pub fn add_validation_rule(&mut self, rule: ValidationRule) {
        self.validation_rules.push(rule);
    }

    pub fn process_dataset(&mut self, dataset_name: &str, data: Vec<f64>) -> Result<Vec<f64>, String> {
        if data.is_empty() {
            return Err("Dataset cannot be empty".to_string());
        }

        for rule in &self.validation_rules {
            if !self.validate_data(&data, rule) {
                return Err(format!("Validation failed for rule: {}", rule.field_name));
            }
        }

        let processed_data = self.transform_data(data);
        self.cache.insert(dataset_name.to_string(), processed_data.clone());
        
        Ok(processed_data)
    }

    pub fn get_cached_data(&self, dataset_name: &str) -> Option<&Vec<f64>> {
        self.cache.get(dataset_name)
    }

    fn validate_data(&self, data: &[f64], rule: &ValidationRule) -> bool {
        if rule.required && data.is_empty() {
            return false;
        }

        for &value in data {
            if value < rule.min_value || value > rule.max_value {
                return false;
            }
        }
        true
    }

    fn transform_data(&self, data: Vec<f64>) -> Vec<f64> {
        let mean = data.iter().sum::<f64>() / data.len() as f64;
        data.into_iter()
            .map(|x| (x - mean).abs())
            .collect()
    }
}

impl ValidationRule {
    pub fn new(field_name: &str, min_value: f64, max_value: f64, required: bool) -> Self {
        ValidationRule {
            field_name: field_name.to_string(),
            min_value,
            max_value,
            required,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_processing() {
        let mut processor = DataProcessor::new();
        let rule = ValidationRule::new("temperature", -50.0, 100.0, true);
        processor.add_validation_rule(rule);

        let test_data = vec![20.5, 25.0, 30.2, 18.7];
        let result = processor.process_dataset("weather", test_data);
        
        assert!(result.is_ok());
        assert_eq!(processor.get_cached_data("weather").unwrap().len(), 4);
    }

    #[test]
    fn test_validation_failure() {
        let mut processor = DataProcessor::new();
        let rule = ValidationRule::new("pressure", 0.0, 10.0, true);
        processor.add_validation_rule(rule);

        let invalid_data = vec![5.0, 15.0, 8.0];
        let result = processor.process_dataset("invalid", invalid_data);
        
        assert!(result.is_err());
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub category: String,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: &str) -> Self {
        Self {
            id,
            value,
            category: category.to_string(),
        }
    }

    pub fn is_valid(&self) -> bool {
        self.id > 0 && self.value >= 0.0 && !self.category.is_empty()
    }
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
        }
    }

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<usize, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut loaded_count = 0;

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            if line_num == 0 || line.trim().is_empty() {
                continue;
            }

            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() != 3 {
                continue;
            }

            let id = match parts[0].parse::<u32>() {
                Ok(val) => val,
                Err(_) => continue,
            };

            let value = match parts[1].parse::<f64>() {
                Ok(val) => val,
                Err(_) => continue,
            };

            let record = DataRecord::new(id, value, parts[2]);
            if record.is_valid() {
                self.records.push(record);
                loaded_count += 1;
            }
        }

        Ok(loaded_count)
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }

        let sum: f64 = self.records.iter().map(|record| record.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn get_statistics(&self) -> (f64, f64, f64) {
        if self.records.is_empty() {
            return (0.0, 0.0, 0.0);
        }

        let values: Vec<f64> = self.records.iter().map(|record| record.value).collect();
        let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let avg = self.calculate_average().unwrap_or(0.0);

        (min, max, avg)
    }

    pub fn count_records(&self) -> usize {
        self.records.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_record_validation() {
        let valid_record = DataRecord::new(1, 10.5, "A");
        assert!(valid_record.is_valid());

        let invalid_id = DataRecord::new(0, 10.5, "A");
        assert!(!invalid_id.is_valid());

        let invalid_value = DataRecord::new(1, -5.0, "A");
        assert!(!invalid_value.is_valid());

        let invalid_category = DataRecord::new(1, 10.5, "");
        assert!(!invalid_category.is_valid());
    }

    #[test]
    fn test_csv_loading() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category").unwrap();
        writeln!(temp_file, "1,10.5,CategoryA").unwrap();
        writeln!(temp_file, "2,15.3,CategoryB").unwrap();
        writeln!(temp_file, "3,8.7,CategoryA").unwrap();

        let mut processor = DataProcessor::new();
        let result = processor.load_from_csv(temp_file.path());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 3);
        assert_eq!(processor.count_records(), 3);
    }

    #[test]
    fn test_filtering() {
        let mut processor = DataProcessor::new();
        processor.records.push(DataRecord::new(1, 10.5, "A"));
        processor.records.push(DataRecord::new(2, 15.3, "B"));
        processor.records.push(DataRecord::new(3, 8.7, "A"));

        let filtered = processor.filter_by_category("A");
        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().all(|r| r.category == "A"));
    }

    #[test]
    fn test_statistics() {
        let mut processor = DataProcessor::new();
        processor.records.push(DataRecord::new(1, 10.0, "A"));
        processor.records.push(DataRecord::new(2, 20.0, "B"));
        processor.records.push(DataRecord::new(3, 30.0, "A"));

        let stats = processor.get_statistics();
        assert_eq!(stats.0, 10.0);
        assert_eq!(stats.1, 30.0);
        assert_eq!(stats.2, 20.0);
    }
}use std::error::Error;
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

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        
        self.data.clear();
        
        for (index, line) in reader.lines().enumerate() {
            let line = line?;
            if index == 0 {
                continue;
            }
            
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() >= 2 {
                if let Ok(value) = parts[1].parse::<f64>() {
                    self.data.push(value);
                }
            }
        }
        
        self.metadata.insert("source".to_string(), file_path.to_string());
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
        
        stats
    }

    pub fn filter_by_threshold(&self, threshold: f64) -> Vec<f64> {
        self.data.iter()
            .filter(|&&x| x > threshold)
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
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        assert_eq!(processor.data_count(), 0);
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value").unwrap();
        writeln!(temp_file, "1,10.5").unwrap();
        writeln!(temp_file, "2,20.3").unwrap();
        writeln!(temp_file, "3,15.7").unwrap();
        
        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(processor.data_count(), 3);
        
        let stats = processor.calculate_statistics();
        assert_eq!(stats["count"], 3.0);
        assert!((stats["mean"] - 15.5).abs() < 0.1);
        
        let filtered = processor.filter_by_threshold(15.0);
        assert_eq!(filtered.len(), 2);
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
    config: ProcessingConfig,
}

pub struct ProcessingConfig {
    pub max_value: f64,
    pub min_value: f64,
    pub allowed_tags: Vec<String>,
}

impl DataProcessor {
    pub fn new(config: ProcessingConfig) -> Self {
        DataProcessor { config }
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if record.value > self.config.max_value {
            return Err(ProcessingError::ValidationError(
                format!("Value {} exceeds maximum {}", record.value, self.config.max_value)
            ));
        }

        if record.value < self.config.min_value {
            return Err(ProcessingError::ValidationError(
                format!("Value {} below minimum {}", record.value, self.config.min_value)
            ));
        }

        for tag in &record.tags {
            if !self.config.allowed_tags.contains(tag) {
                return Err(ProcessingError::ValidationError(
                    format!("Tag '{}' is not allowed", tag)
                ));
            }
        }

        Ok(())
    }

    pub fn transform_record(&self, record: &DataRecord) -> Result<HashMap<String, String>, ProcessingError> {
        let mut result = HashMap::new();
        
        result.insert("id".to_string(), record.id.to_string());
        result.insert("name".to_string(), record.name.to_uppercase());
        result.insert("value".to_string(), format!("{:.2}", record.value));
        result.insert("tags_count".to_string(), record.tags.len().to_string());
        result.insert("tags".to_string(), record.tags.join(","));
        
        if record.value < 0.0 {
            return Err(ProcessingError::TransformationFailed(
                "Negative values cannot be transformed".to_string()
            ));
        }

        Ok(result)
    }

    pub fn process_batch(&self, records: Vec<DataRecord>) -> Result<Vec<HashMap<String, String>>, ProcessingError> {
        let mut processed = Vec::new();
        
        for record in records {
            self.validate_record(&record)?;
            let transformed = self.transform_record(&record)?;
            processed.push(transformed);
        }
        
        Ok(processed)
    }
}

pub fn calculate_statistics(records: &[DataRecord]) -> (f64, f64, f64) {
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

    fn create_test_config() -> ProcessingConfig {
        ProcessingConfig {
            max_value: 1000.0,
            min_value: 0.0,
            allowed_tags: vec!["alpha".to_string(), "beta".to_string(), "gamma".to_string()],
        }
    }

    #[test]
    fn test_validation_success() {
        let processor = DataProcessor::new(create_test_config());
        let record = DataRecord {
            id: 1,
            name: "test".to_string(),
            value: 500.0,
            tags: vec!["alpha".to_string(), "beta".to_string()],
        };
        
        assert!(processor.validate_record(&record).is_ok());
    }

    #[test]
    fn test_validation_fails_on_invalid_tag() {
        let processor = DataProcessor::new(create_test_config());
        let record = DataRecord {
            id: 1,
            name: "test".to_string(),
            value: 500.0,
            tags: vec!["invalid".to_string()],
        };
        
        assert!(processor.validate_record(&record).is_err());
    }

    #[test]
    fn test_transform_record() {
        let processor = DataProcessor::new(create_test_config());
        let record = DataRecord {
            id: 42,
            name: "sample".to_string(),
            value: 123.456,
            tags: vec!["alpha".to_string()],
        };
        
        let result = processor.transform_record(&record).unwrap();
        
        assert_eq!(result.get("id").unwrap(), "42");
        assert_eq!(result.get("name").unwrap(), "SAMPLE");
        assert_eq!(result.get("value").unwrap(), "123.46");
        assert_eq!(result.get("tags_count").unwrap(), "1");
    }

    #[test]
    fn test_calculate_statistics() {
        let records = vec![
            DataRecord { id: 1, name: "a".to_string(), value: 10.0, tags: vec![] },
            DataRecord { id: 2, name: "b".to_string(), value: 20.0, tags: vec![] },
            DataRecord { id: 3, name: "c".to_string(), value: 30.0, tags: vec![] },
        ];
        
        let (mean, variance, std_dev) = calculate_statistics(&records);
        
        assert_eq!(mean, 20.0);
        assert_eq!(variance, 66.66666666666667);
        assert_eq!(std_dev, 8.16496580927726);
    }
}
use std::collections::HashMap;

pub struct DataProcessor {
    data: HashMap<String, Vec<f64>>,
    validation_rules: HashMap<String, ValidationRule>,
}

pub struct ValidationRule {
    min_value: Option<f64>,
    max_value: Option<f64>,
    required: bool,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            data: HashMap::new(),
            validation_rules: HashMap::new(),
        }
    }

    pub fn add_dataset(&mut self, key: &str, values: Vec<f64>) {
        self.data.insert(key.to_string(), values);
    }

    pub fn set_validation_rule(&mut self, key: &str, rule: ValidationRule) {
        self.validation_rules.insert(key.to_string(), rule);
    }

    pub fn validate_dataset(&self, key: &str) -> Result<(), String> {
        let data = match self.data.get(key) {
            Some(d) => d,
            None => return Err(format!("Dataset '{}' not found", key)),
        };

        let rule = match self.validation_rules.get(key) {
            Some(r) => r,
            None => return Ok(()),
        };

        if rule.required && data.is_empty() {
            return Err(format!("Dataset '{}' is required but empty", key));
        }

        for &value in data {
            if let Some(min) = rule.min_value {
                if value < min {
                    return Err(format!("Value {} below minimum {} in dataset '{}'", value, min, key));
                }
            }

            if let Some(max) = rule.max_value {
                if value > max {
                    return Err(format!("Value {} above maximum {} in dataset '{}'", value, max, key));
                }
            }
        }

        Ok(())
    }

    pub fn calculate_statistics(&self, key: &str) -> Option<DatasetStatistics> {
        let data = self.data.get(key)?;

        if data.is_empty() {
            return None;
        }

        let sum: f64 = data.iter().sum();
        let count = data.len() as f64;
        let mean = sum / count;

        let variance: f64 = data.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / count;

        let sorted_data = {
            let mut sorted = data.clone();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
            sorted
        };

        let median = if count as usize % 2 == 0 {
            let mid = count as usize / 2;
            (sorted_data[mid - 1] + sorted_data[mid]) / 2.0
        } else {
            sorted_data[count as usize / 2]
        };

        Some(DatasetStatistics {
            count: data.len(),
            mean,
            variance,
            standard_deviation: variance.sqrt(),
            median,
            min: *sorted_data.first().unwrap(),
            max: *sorted_data.last().unwrap(),
        })
    }

    pub fn normalize_data(&mut self, key: &str) -> Result<(), String> {
        let stats = match self.calculate_statistics(key) {
            Some(s) => s,
            None => return Err(format!("Cannot calculate statistics for dataset '{}'", key)),
        };

        if stats.standard_deviation == 0.0 {
            return Err(format!("Standard deviation is zero for dataset '{}'", key));
        }

        if let Some(data) = self.data.get_mut(key) {
            for value in data.iter_mut() {
                *value = (*value - stats.mean) / stats.standard_deviation;
            }
        }

        Ok(())
    }
}

pub struct DatasetStatistics {
    pub count: usize,
    pub mean: f64,
    pub variance: f64,
    pub standard_deviation: f64,
    pub median: f64,
    pub min: f64,
    pub max: f64,
}

impl ValidationRule {
    pub fn new() -> Self {
        ValidationRule {
            min_value: None,
            max_value: None,
            required: false,
        }
    }

    pub fn with_min(mut self, min: f64) -> Self {
        self.min_value = Some(min);
        self
    }

    pub fn with_max(mut self, max: f64) -> Self {
        self.max_value = Some(max);
        self
    }

    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        
        processor.add_dataset("temperatures", vec![20.5, 22.1, 19.8, 23.4, 21.9]);
        
        let rule = ValidationRule::new()
            .with_min(15.0)
            .with_max(30.0)
            .required();
        
        processor.set_validation_rule("temperatures", rule);
        
        assert!(processor.validate_dataset("temperatures").is_ok());
        
        let stats = processor.calculate_statistics("temperatures").unwrap();
        assert_eq!(stats.count, 5);
        assert!(stats.mean > 20.0 && stats.mean < 23.0);
        
        assert!(processor.normalize_data("temperatures").is_ok());
    }

    #[test]
    fn test_validation_failure() {
        let mut processor = DataProcessor::new();
        processor.add_dataset("invalid_data", vec![10.0, 50.0]);
        
        let rule = ValidationRule::new()
            .with_max(30.0)
            .required();
        
        processor.set_validation_rule("invalid_data", rule);
        
        assert!(processor.validate_dataset("invalid_data").is_err());
    }
}