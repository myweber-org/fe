
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

pub fn create_default_processor() -> DataProcessor {
    let mut processor = DataProcessor::new();

    processor.add_validation_rule(|record| {
        if record.values.is_empty() {
            Err(ProcessingError::ValidationError("Values cannot be empty".to_string()))
        } else {
            Ok(())
        }
    });

    processor.add_validation_rule(|record| {
        if record.timestamp < 0 {
            Err(ProcessingError::ValidationError("Timestamp must be non-negative".to_string()))
        } else {
            Ok(())
        }
    });

    processor.add_transformation(|mut record| {
        let sum: f64 = record.values.iter().sum();
        let count = record.values.len() as f64;
        record.metadata.insert("average".to_string(), (sum / count).to_string());
        Ok(record)
    });

    processor.add_transformation(|mut record| {
        record.values = record.values.into_iter().map(|v| v * 2.0).collect();
        Ok(record)
    });

    processor
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_processing() {
        let processor = create_default_processor();
        
        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), "test".to_string());
        
        let record = DataRecord {
            id: 1,
            timestamp: 1625097600,
            values: vec![1.0, 2.0, 3.0],
            metadata,
        };

        let result = processor.process(record);
        assert!(result.is_ok());
        
        let processed = result.unwrap();
        assert_eq!(processed.values, vec![2.0, 4.0, 6.0]);
        assert!(processed.metadata.contains_key("average"));
    }

    #[test]
    fn test_validation_failure() {
        let processor = create_default_processor();
        
        let record = DataRecord {
            id: 2,
            timestamp: -1,
            values: vec![1.0, 2.0],
            metadata: HashMap::new(),
        };

        let result = processor.process(record);
        assert!(result.is_err());
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
        let values: Vec<f64> = records
            .iter()
            .filter_map(|record| record.get(column_index))
            .filter_map(|s| s.parse::<f64>().ok())
            .collect();

        if values.is_empty() {
            return None;
        }

        let sum: f64 = values.iter().sum();
        let count = values.len() as f64;
        let mean = sum / count;
        
        let variance: f64 = values
            .iter()
            .map(|value| {
                let diff = mean - value;
                diff * diff
            })
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
    fn test_data_processor() {
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
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub category: String,
}

impl DataRecord {
    pub fn new(id: u32, name: String, value: f64, category: String) -> Self {
        Self {
            id,
            name,
            value,
            category,
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.name.is_empty() && self.value >= 0.0 && !self.category.is_empty()
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
        let mut count = 0;

        for (line_num, line) in reader.lines().enumerate() {
            if line_num == 0 {
                continue;
            }

            let line = line?;
            let parts: Vec<&str> = line.split(',').collect();
            
            if parts.len() != 4 {
                continue;
            }

            let id = match parts[0].parse::<u32>() {
                Ok(val) => val,
                Err(_) => continue,
            };

            let name = parts[1].to_string();
            let value = match parts[2].parse::<f64>() {
                Ok(val) => val,
                Err(_) => continue,
            };

            let category = parts[3].to_string();

            let record = DataRecord::new(id, name, value, category);
            if record.is_valid() {
                self.records.push(record);
                count += 1;
            }
        }

        Ok(count)
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

    pub fn get_records(&self) -> &[DataRecord] {
        &self.records
    }

    pub fn clear(&mut self) {
        self.records.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_record_validation() {
        let valid_record = DataRecord::new(1, "Test".to_string(), 10.5, "A".to_string());
        assert!(valid_record.is_valid());

        let invalid_record = DataRecord::new(2, "".to_string(), -5.0, "B".to_string());
        assert!(!invalid_record.is_valid());
    }

    #[test]
    fn test_load_from_csv() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,category").unwrap();
        writeln!(temp_file, "1,Item1,10.5,CategoryA").unwrap();
        writeln!(temp_file, "2,Item2,20.0,CategoryB").unwrap();
        writeln!(temp_file, "3,Item3,15.75,CategoryA").unwrap();

        let mut processor = DataProcessor::new();
        let result = processor.load_from_csv(temp_file.path());
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 3);
        assert_eq!(processor.get_records().len(), 3);
    }

    #[test]
    fn test_filter_by_category() {
        let mut processor = DataProcessor::new();
        processor.records.push(DataRecord::new(1, "A".to_string(), 10.0, "X".to_string()));
        processor.records.push(DataRecord::new(2, "B".to_string(), 20.0, "Y".to_string()));
        processor.records.push(DataRecord::new(3, "C".to_string(), 30.0, "X".to_string()));

        let filtered = processor.filter_by_category("X");
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_calculate_average() {
        let mut processor = DataProcessor::new();
        assert!(processor.calculate_average().is_none());

        processor.records.push(DataRecord::new(1, "A".to_string(), 10.0, "X".to_string()));
        processor.records.push(DataRecord::new(2, "B".to_string(), 20.0, "Y".to_string()));
        
        let avg = processor.calculate_average();
        assert!(avg.is_some());
        assert_eq!(avg.unwrap(), 15.0);
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
    active: bool,
}

pub fn process_data_file(file_path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
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
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_process_valid_data() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,active").unwrap();
        writeln!(temp_file, "1,Test1,10.5,true").unwrap();
        writeln!(temp_file, "2,Test2,-3.2,false").unwrap();
        writeln!(temp_file, "3,Test3,7.8,true").unwrap();

        let records = process_data_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].name, "Test1");
        assert_eq!(records[1].name, "Test3");
    }

    #[test]
    fn test_calculate_statistics() {
        let records = vec![
            Record { id: 1, name: "A".to_string(), value: 10.0, active: true },
            Record { id: 2, name: "B".to_string(), value: 20.0, active: false },
            Record { id: 3, name: "C".to_string(), value: 30.0, active: true },
        ];

        let (mean, variance, std_dev) = calculate_statistics(&records);
        assert_eq!(mean, 20.0);
        assert_eq!(variance, 66.66666666666667);
        assert_eq!(std_dev, 8.16496580927726);
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub category: String,
    pub timestamp: String,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: String, timestamp: String) -> Self {
        Self {
            id,
            value,
            category,
            timestamp,
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.category.is_empty() 
            && self.value.is_finite() 
            && self.id > 0
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
        let mut count = 0;

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            
            if line_num == 0 {
                continue;
            }

            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() != 4 {
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

            let category = parts[2].to_string();
            let timestamp = parts[3].to_string();

            let record = DataRecord::new(id, value, category, timestamp);
            if record.is_valid() {
                self.records.push(record);
                count += 1;
            }
        }

        Ok(count)
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

        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn get_statistics(&self) -> (f64, f64, f64) {
        if self.records.is_empty() {
            return (0.0, 0.0, 0.0);
        }

        let values: Vec<f64> = self.records.iter().map(|r| r.value).collect();
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
        let valid_record = DataRecord::new(1, 42.5, "A".to_string(), "2023-01-01".to_string());
        assert!(valid_record.is_valid());

        let invalid_record = DataRecord::new(0, f64::NAN, "".to_string(), "2023-01-01".to_string());
        assert!(!invalid_record.is_valid());
    }

    #[test]
    fn test_data_processing() {
        let mut processor = DataProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category,timestamp").unwrap();
        writeln!(temp_file, "1,10.5,A,2023-01-01").unwrap();
        writeln!(temp_file, "2,20.0,B,2023-01-02").unwrap();
        writeln!(temp_file, "3,15.3,A,2023-01-03").unwrap();

        let count = processor.load_from_csv(temp_file.path()).unwrap();
        assert_eq!(count, 3);
        assert_eq!(processor.count_records(), 3);

        let category_a = processor.filter_by_category("A");
        assert_eq!(category_a.len(), 2);

        let stats = processor.get_statistics();
        assert_eq!(stats.0, 10.5);
        assert_eq!(stats.1, 20.0);
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct DataRecord {
    id: u32,
    value: f64,
    category: String,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: String) -> Result<Self, String> {
        if value < 0.0 {
            return Err("Value cannot be negative".to_string());
        }
        if category.is_empty() {
            return Err("Category cannot be empty".to_string());
        }
        Ok(Self { id, value, category })
    }

    pub fn calculate_adjusted_value(&self, multiplier: f64) -> f64 {
        self.value * multiplier
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

        for (line_num, line) in reader.lines().enumerate().skip(1) {
            let line = line?;
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

            let category = parts[2].to_string();

            match DataRecord::new(id, value, category) {
                Ok(record) => {
                    self.records.push(record);
                    count += 1;
                }
                Err(_) => {
                    eprintln!("Warning: Invalid record at line {}", line_num + 1);
                }
            }
        }

        Ok(count)
    }

    pub fn total_value(&self) -> f64 {
        self.records.iter().map(|r| r.value).sum()
    }

    pub fn average_value(&self) -> Option<f64> {
        if self.records.is_empty() {
            None
        } else {
            Some(self.total_value() / self.records.len() as f64)
        }
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
        let record = DataRecord::new(1, 42.5, "test".to_string());
        assert!(record.is_ok());
        
        let record = DataRecord::new(2, -10.0, "test".to_string());
        assert!(record.is_err());
    }

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        assert_eq!(processor.record_count(), 0);
        assert_eq!(processor.total_value(), 0.0);
        assert_eq!(processor.average_value(), None);
    }

    #[test]
    fn test_csv_loading() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category").unwrap();
        writeln!(temp_file, "1,100.5,alpha").unwrap();
        writeln!(temp_file, "2,200.0,beta").unwrap();
        writeln!(temp_file, "3,invalid,gamma").unwrap();
        
        let mut processor = DataProcessor::new();
        let result = processor.load_from_csv(temp_file.path());
        
        assert!(result.is_ok());
        assert_eq!(processor.record_count(), 2);
        assert_eq!(processor.total_value(), 300.5);
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;

pub struct DataProcessor {
    records: Vec<HashMap<String, String>>,
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
                let values: Vec<String> = line.split(',').map(|s| s.trim().to_string()).collect();
                
                if values.len() == headers.len() {
                    let mut record = HashMap::new();
                    for (i, header) in headers.iter().enumerate() {
                        record.insert(header.clone(), values[i].clone());
                    }
                    self.records.push(record);
                }
            }
        }
        
        Ok(())
    }

    pub fn calculate_statistics(&self, column_name: &str) -> Option<Statistics> {
        let mut numeric_values = Vec::new();
        
        for record in &self.records {
            if let Some(value_str) = record.get(column_name) {
                if let Ok(value) = value_str.parse::<f64>() {
                    numeric_values.push(value);
                }
            }
        }
        
        if numeric_values.is_empty() {
            return None;
        }
        
        let count = numeric_values.len();
        let sum: f64 = numeric_values.iter().sum();
        let mean = sum / count as f64;
        
        let variance: f64 = numeric_values.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / count as f64;
        
        let std_dev = variance.sqrt();
        
        Some(Statistics {
            count,
            sum,
            mean,
            variance,
            std_dev,
        })
    }

    pub fn filter_records<F>(&self, predicate: F) -> Vec<HashMap<String, String>>
    where
        F: Fn(&HashMap<String, String>) -> bool,
    {
        self.records.iter()
            .filter(|record| predicate(record))
            .cloned()
            .collect()
    }

    pub fn get_unique_values(&self, column_name: &str) -> Vec<String> {
        let mut unique_values = std::collections::HashSet::new();
        
        for record in &self.records {
            if let Some(value) = record.get(column_name) {
                unique_values.insert(value.clone());
            }
        }
        
        unique_values.into_iter().collect()
    }

    pub fn record_count(&self) -> usize {
        self.records.len()
    }
}

pub struct Statistics {
    pub count: usize,
    pub sum: f64,
    pub mean: f64,
    pub variance: f64,
    pub std_dev: f64,
}

impl std::fmt::Display for Statistics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Count: {}, Sum: {:.2}, Mean: {:.2}, Variance: {:.2}, Std Dev: {:.2}",
            self.count, self.sum, self.mean, self.variance, self.std_dev
        )
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

    pub fn process_dataset(&mut self, key: &str, data: &[f64]) -> Result<Vec<f64>, String> {
        if data.is_empty() {
            return Err("Empty dataset provided".to_string());
        }

        if let Some(cached) = self.cache.get(key) {
            return Ok(cached.clone());
        }

        let validated = self.validate_data(data)?;
        let normalized = self.normalize_data(&validated);
        let transformed = self.apply_transformations(&normalized);

        self.cache.insert(key.to_string(), transformed.clone());
        Ok(transformed)
    }

    fn validate_data(&self, data: &[f64]) -> Result<Vec<f64>, String> {
        let mut result = Vec::with_capacity(data.len());
        
        for &value in data {
            if !value.is_finite() {
                return Err(format!("Invalid numeric value encountered: {}", value));
            }
            if value < 0.0 {
                return Err("Negative values not allowed in this context".to_string());
            }
            result.push(value);
        }
        
        Ok(result)
    }

    fn normalize_data(&self, data: &[f64]) -> Vec<f64> {
        let sum: f64 = data.iter().sum();
        if sum == 0.0 {
            return vec![0.0; data.len()];
        }
        
        data.iter()
            .map(|&x| x / sum)
            .collect()
    }

    fn apply_transformations(&self, data: &[f64]) -> Vec<f64> {
        data.iter()
            .map(|&x| (x * 100.0).ln_1p())
            .collect()
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    pub fn get_cache_stats(&self) -> (usize, usize) {
        let total_items: usize = self.cache.values().map(|v| v.len()).sum();
        (self.cache.len(), total_items)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_validation() {
        let processor = DataProcessor::new();
        let valid_data = vec![1.0, 2.0, 3.0];
        let invalid_data = vec![1.0, f64::NAN, 3.0];
        
        assert!(processor.validate_data(&valid_data).is_ok());
        assert!(processor.validate_data(&invalid_data).is_err());
    }

    #[test]
    fn test_normalization() {
        let processor = DataProcessor::new();
        let data = vec![1.0, 2.0, 3.0];
        let normalized = processor.normalize_data(&data);
        
        let sum: f64 = normalized.iter().sum();
        assert!((sum - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_cache_functionality() {
        let mut processor = DataProcessor::new();
        let data = vec![10.0, 20.0, 30.0];
        
        let result1 = processor.process_dataset("test", &data).unwrap();
        let result2 = processor.process_dataset("test", &data).unwrap();
        
        assert_eq!(result1, result2);
        assert_eq!(processor.get_cache_stats(), (1, 1));
        
        processor.clear_cache();
        assert_eq!(processor.get_cache_stats(), (0, 0));
    }
}