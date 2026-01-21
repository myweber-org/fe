
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
pub enum ValidationError {
    InvalidId,
    EmptyValues,
    ValueOutOfRange(f64),
    MissingMetadata(String),
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::InvalidId => write!(f, "Invalid record ID"),
            ValidationError::EmptyValues => write!(f, "Record values cannot be empty"),
            ValidationError::ValueOutOfRange(val) => write!(f, "Value {} out of valid range", val),
            ValidationError::MissingMetadata(key) => write!(f, "Missing metadata key: {}", key),
        }
    }
}

impl Error for ValidationError {}

pub struct DataProcessor {
    min_value: f64,
    max_value: f64,
    required_metadata: Vec<String>,
}

impl DataProcessor {
    pub fn new(min_value: f64, max_value: f64, required_metadata: Vec<String>) -> Self {
        DataProcessor {
            min_value,
            max_value,
            required_metadata,
        }
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), ValidationError> {
        if record.id == 0 {
            return Err(ValidationError::InvalidId);
        }

        if record.values.is_empty() {
            return Err(ValidationError::EmptyValues);
        }

        for &value in &record.values {
            if value < self.min_value || value > self.max_value {
                return Err(ValidationError::ValueOutOfRange(value));
            }
        }

        for key in &self.required_metadata {
            if !record.metadata.contains_key(key) {
                return Err(ValidationError::MissingMetadata(key.clone()));
            }
        }

        Ok(())
    }

    pub fn normalize_values(&self, record: &mut DataRecord) {
        let min_val = record.values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max_val = record.values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        
        if max_val > min_val {
            for value in &mut record.values {
                *value = (*value - min_val) / (max_val - min_val);
            }
        }
    }

    pub fn process_records(&self, records: &mut [DataRecord]) -> Vec<Result<(), ValidationError>> {
        let mut results = Vec::new();
        
        for record in records {
            match self.validate_record(record) {
                Ok(_) => {
                    self.normalize_values(record);
                    results.push(Ok(()));
                }
                Err(e) => {
                    results.push(Err(e));
                }
            }
        }
        
        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_record() -> DataRecord {
        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), "test".to_string());
        metadata.insert("version".to_string(), "1.0".to_string());
        
        DataRecord {
            id: 1,
            values: vec![10.0, 20.0, 30.0],
            metadata,
        }
    }

    #[test]
    fn test_valid_record() {
        let processor = DataProcessor::new(
            0.0,
            100.0,
            vec!["source".to_string(), "version".to_string()]
        );
        
        let record = create_test_record();
        assert!(processor.validate_record(&record).is_ok());
    }

    #[test]
    fn test_invalid_id() {
        let processor = DataProcessor::new(
            0.0,
            100.0,
            vec!["source".to_string()]
        );
        
        let mut record = create_test_record();
        record.id = 0;
        
        assert!(matches!(
            processor.validate_record(&record),
            Err(ValidationError::InvalidId)
        ));
    }

    #[test]
    fn test_normalize_values() {
        let processor = DataProcessor::new(
            0.0,
            100.0,
            vec!["source".to_string()]
        );
        
        let mut record = create_test_record();
        processor.normalize_values(&mut record);
        
        assert_eq!(record.values[0], 0.0);
        assert_eq!(record.values[1], 0.5);
        assert_eq!(record.values[2], 1.0);
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
    pub category: String,
}

#[derive(Debug)]
pub enum ValidationError {
    InvalidId,
    EmptyName,
    NegativeValue,
    UnknownCategory,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::InvalidId => write!(f, "ID must be greater than zero"),
            ValidationError::EmptyName => write!(f, "Name cannot be empty"),
            ValidationError::NegativeValue => write!(f, "Value cannot be negative"),
            ValidationError::UnknownCategory => write!(f, "Category is not recognized"),
        }
    }
}

impl Error for ValidationError {}

pub struct DataProcessor {
    valid_categories: Vec<String>,
    transformation_rules: HashMap<String, f64>,
}

impl DataProcessor {
    pub fn new() -> Self {
        let mut processor = DataProcessor {
            valid_categories: Vec::new(),
            transformation_rules: HashMap::new(),
        };
        
        processor.valid_categories.extend(vec![
            "standard".to_string(),
            "premium".to_string(),
            "economy".to_string(),
        ]);
        
        processor.transformation_rules.insert("standard".to_string(), 1.0);
        processor.transformation_rules.insert("premium".to_string(), 1.2);
        processor.transformation_rules.insert("economy".to_string(), 0.8);
        
        processor
    }
    
    pub fn validate_record(&self, record: &DataRecord) -> Result<(), ValidationError> {
        if record.id == 0 {
            return Err(ValidationError::InvalidId);
        }
        
        if record.name.trim().is_empty() {
            return Err(ValidationError::EmptyName);
        }
        
        if record.value < 0.0 {
            return Err(ValidationError::NegativeValue);
        }
        
        if !self.valid_categories.contains(&record.category) {
            return Err(ValidationError::UnknownCategory);
        }
        
        Ok(())
    }
    
    pub fn transform_value(&self, record: &DataRecord) -> f64 {
        match self.transformation_rules.get(&record.category) {
            Some(multiplier) => record.value * multiplier,
            None => record.value,
        }
    }
    
    pub fn process_batch(&self, records: Vec<DataRecord>) -> Vec<Result<f64, ValidationError>> {
        records
            .iter()
            .map(|record| {
                self.validate_record(record)
                    .map(|_| self.transform_value(record))
            })
            .collect()
    }
    
    pub fn add_category(&mut self, category: String, multiplier: f64) {
        if !self.valid_categories.contains(&category) {
            self.valid_categories.push(category.clone());
            self.transformation_rules.insert(category, multiplier);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_valid_record() {
        let processor = DataProcessor::new();
        let record = DataRecord {
            id: 1,
            name: "Test Record".to_string(),
            value: 100.0,
            category: "standard".to_string(),
        };
        
        assert!(processor.validate_record(&record).is_ok());
        assert_eq!(processor.transform_value(&record), 100.0);
    }
    
    #[test]
    fn test_invalid_id() {
        let processor = DataProcessor::new();
        let record = DataRecord {
            id: 0,
            name: "Test Record".to_string(),
            value: 100.0,
            category: "standard".to_string(),
        };
        
        assert!(matches!(processor.validate_record(&record), Err(ValidationError::InvalidId)));
    }
    
    #[test]
    fn test_premium_multiplier() {
        let processor = DataProcessor::new();
        let record = DataRecord {
            id: 1,
            name: "Premium Item".to_string(),
            value: 100.0,
            category: "premium".to_string(),
        };
        
        assert!(processor.validate_record(&record).is_ok());
        assert_eq!(processor.transform_value(&record), 120.0);
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

    pub fn filter_by_category(&self, category: &str) -> Vec<&Record> {
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

    pub fn validate_records(&self) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|record| record.value < 0.0 || record.name.is_empty())
            .collect()
    }

    pub fn get_statistics(&self) -> (usize, Option<f64>, Option<f64>) {
        let count = self.records.len();
        let min = self.records.iter().map(|r| r.value).reduce(f64::min);
        let max = self.records.iter().map(|r| r.value).reduce(f64::max);

        (count, min, max)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        
        let csv_data = "id,name,value,category\n1,ItemA,10.5,Alpha\n2,ItemB,-5.0,Beta\n3,ItemC,15.0,Alpha\n";
        
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", csv_data).unwrap();
        
        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        
        let alpha_records = processor.filter_by_category("Alpha");
        assert_eq!(alpha_records.len(), 2);
        
        let invalid_records = processor.validate_records();
        assert_eq!(invalid_records.len(), 1);
        
        let stats = processor.get_statistics();
        assert_eq!(stats.0, 3);
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

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        
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
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value").unwrap();
        writeln!(temp_file, "1,10.5").unwrap();
        writeln!(temp_file, "2,20.3").unwrap();
        writeln!(temp_file, "3,15.7").unwrap();
        
        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(processor.data_count(), 3);
        
        let stats = processor.calculate_statistics();
        assert!((stats["mean"] - 15.5).abs() < 0.1);
        assert!((stats["std_dev"] - 4.9).abs() < 0.1);
        
        let filtered = processor.filter_data(15.0);
        assert_eq!(filtered.len(), 2);
        
        let metadata = processor.get_metadata();
        assert!(metadata.contains_key("source"));
    }
}use std::error::Error;
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
            let fields: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();
            
            if !fields.is_empty() {
                records.push(fields);
            }
        }

        Ok(records)
    }

    pub fn validate_record(&self, record: &[String]) -> bool {
        !record.is_empty() && record.iter().all(|field| !field.is_empty())
    }

    pub fn calculate_statistics(&self, records: &[Vec<String>], column_index: usize) -> Option<(f64, f64, f64)> {
        let mut values = Vec::new();
        
        for record in records {
            if column_index < record.len() {
                if let Ok(value) = record[column_index].parse::<f64>() {
                    values.push(value);
                }
            }
        }

        if values.is_empty() {
            return None;
        }

        let sum: f64 = values.iter().sum();
        let count = values.len() as f64;
        let mean = sum / count;
        
        let variance: f64 = values.iter()
            .map(|v| (v - mean).powi(2))
            .sum::<f64>() / count;
        
        let std_dev = variance.sqrt();

        Some((mean, variance, std_dev))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,salary").unwrap();
        writeln!(temp_file, "Alice,30,50000.0").unwrap();
        writeln!(temp_file, "Bob,25,45000.0").unwrap();
        writeln!(temp_file, "Charlie,35,55000.0").unwrap();

        let processor = DataProcessor::new(',', true);
        let records = processor.process_file(temp_file.path()).unwrap();
        
        assert_eq!(records.len(), 3);
        assert!(processor.validate_record(&records[0]));
        
        let stats = processor.calculate_statistics(&records, 1);
        assert!(stats.is_some());
        
        let (mean, _, _) = stats.unwrap();
        assert!((mean - 30.0).abs() < 0.001);
    }
}