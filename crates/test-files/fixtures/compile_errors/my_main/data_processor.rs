
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
            ValidationError::ValueOutOfRange(val) => write!(f, "Value {} is out of valid range", val),
            ValidationError::MissingMetadata(key) => write!(f, "Missing required metadata: {}", key),
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

        for required_key in &self.required_metadata {
            if !record.metadata.contains_key(required_key) {
                return Err(ValidationError::MissingMetadata(required_key.clone()));
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
                Ok(()) => {
                    self.normalize_values(record);
                    results.push(Ok(()));
                }
                Err(err) => {
                    results.push(Err(err));
                }
            }
        }
        
        results
    }

    pub fn calculate_statistics(&self, records: &[DataRecord]) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        
        if records.is_empty() {
            return stats;
        }

        let value_count = records[0].values.len();
        let mut sums = vec![0.0; value_count];
        let mut squares = vec![0.0; value_count];
        
        for record in records {
            for (i, &value) in record.values.iter().enumerate() {
                sums[i] += value;
                squares[i] += value * value;
            }
        }

        let record_count = records.len() as f64;
        
        for i in 0..value_count {
            let mean = sums[i] / record_count;
            let variance = (squares[i] / record_count) - (mean * mean);
            
            stats.insert(format!("mean_{}", i), mean);
            stats.insert(format!("variance_{}", i), variance);
            stats.insert(format!("std_dev_{}", i), variance.sqrt());
        }

        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_record() -> DataRecord {
        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), "test".to_string());
        
        DataRecord {
            id: 1,
            values: vec![10.0, 20.0, 30.0],
            metadata,
        }
    }

    #[test]
    fn test_valid_record() {
        let processor = DataProcessor::new(0.0, 100.0, vec!["source".to_string()]);
        let record = create_test_record();
        
        assert!(processor.validate_record(&record).is_ok());
    }

    #[test]
    fn test_invalid_id() {
        let processor = DataProcessor::new(0.0, 100.0, vec!["source".to_string()]);
        let mut record = create_test_record();
        record.id = 0;
        
        assert!(matches!(
            processor.validate_record(&record),
            Err(ValidationError::InvalidId)
        ));
    }

    #[test]
    fn test_normalization() {
        let processor = DataProcessor::new(0.0, 100.0, vec!["source".to_string()]);
        let mut record = create_test_record();
        
        processor.normalize_values(&mut record);
        
        assert_eq!(record.values[0], 0.0);
        assert_eq!(record.values[1], 0.5);
        assert_eq!(record.values[2], 1.0);
    }

    #[test]
    fn test_statistics_calculation() {
        let processor = DataProcessor::new(0.0, 100.0, vec!["source".to_string()]);
        let records = vec![create_test_record(), create_test_record()];
        
        let stats = processor.calculate_statistics(&records);
        
        assert_eq!(stats.get("mean_0"), Some(&10.0));
        assert_eq!(stats.get("variance_0"), Some(&0.0));
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;

pub struct DataProcessor {
    data: Vec<f64>,
    frequency_map: HashMap<String, u32>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            data: Vec::new(),
            frequency_map: HashMap::new(),
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
                
                let category = parts[0].to_string();
                *self.frequency_map.entry(category).or_insert(0) += 1;
            }
        }
        
        Ok(())
    }

    pub fn calculate_mean(&self) -> Option<f64> {
        if self.data.is_empty() {
            return None;
        }
        
        let sum: f64 = self.data.iter().sum();
        Some(sum / self.data.len() as f64)
    }

    pub fn calculate_median(&mut self) -> Option<f64> {
        if self.data.is_empty() {
            return None;
        }
        
        self.data.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let mid = self.data.len() / 2;
        
        if self.data.len() % 2 == 0 {
            Some((self.data[mid - 1] + self.data[mid]) / 2.0)
        } else {
            Some(self.data[mid])
        }
    }

    pub fn get_frequency_distribution(&self) -> &HashMap<String, u32> {
        &self.frequency_map
    }

    pub fn filter_by_threshold(&self, threshold: f64) -> Vec<f64> {
        self.data.iter()
            .filter(|&&x| x > threshold)
            .cloned()
            .collect()
    }

    pub fn data_summary(&self) -> String {
        let mean = self.calculate_mean().unwrap_or(0.0);
        let count = self.data.len();
        let unique_categories = self.frequency_map.len();
        
        format!(
            "Data Summary: {} records, {:.2} mean, {} categories",
            count, mean, unique_categories
        )
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
        writeln!(temp_file, "category,value").unwrap();
        writeln!(temp_file, "A,10.5").unwrap();
        writeln!(temp_file, "B,20.3").unwrap();
        writeln!(temp_file, "A,15.7").unwrap();
        
        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        
        assert_eq!(processor.calculate_mean(), Some(15.5));
        assert_eq!(processor.filter_by_threshold(15.0).len(), 2);
        
        let freq = processor.get_frequency_distribution();
        assert_eq!(freq.get("A"), Some(&2));
        assert_eq!(freq.get("B"), Some(&1));
    }
}
use csv::Reader;
use serde::Deserialize;
use std::error::Error;

#[derive(Debug, Deserialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

pub fn process_csv_data(file_path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let mut reader = Reader::from_path(file_path)?;
    let mut records = Vec::new();

    for result in reader.deserialize() {
        let record: Record = result?;
        validate_record(&record)?;
        records.push(record);
    }

    Ok(records)
}

fn validate_record(record: &Record) -> Result<(), Box<dyn Error>> {
    if record.name.is_empty() {
        return Err("Name cannot be empty".into());
    }
    if record.value < 0.0 {
        return Err("Value must be non-negative".into());
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

    let variance: f64 = records
        .iter()
        .map(|r| (r.value - mean).powi(2))
        .sum::<f64>() / count;

    let std_dev = variance.sqrt();

    (sum, mean, std_dev)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_process_valid_csv() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "id,name,value,category").unwrap();
        writeln!(file, "1,Test1,100.5,A").unwrap();
        writeln!(file, "2,Test2,200.0,B").unwrap();

        let result = process_csv_data(file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 2);
    }

    #[test]
    fn test_calculate_statistics() {
        let records = vec![
            Record { id: 1, name: "A".to_string(), value: 10.0, category: "X".to_string() },
            Record { id: 2, name: "B".to_string(), value: 20.0, category: "Y".to_string() },
            Record { id: 3, name: "C".to_string(), value: 30.0, category: "Z".to_string() },
        ];

        let (sum, mean, std_dev) = calculate_statistics(&records);
        assert_eq!(sum, 60.0);
        assert_eq!(mean, 20.0);
        assert!((std_dev - 8.164965).abs() < 0.0001);
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, PartialEq)]
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
        let reader = BufReader::new(file);
        
        let mut count = 0;
        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            
            if line_num == 0 {
                continue;
            }
            
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() != 3 {
                return Err(format!("Invalid CSV format at line {}", line_num + 1).into());
            }
            
            let id = parts[0].parse::<u32>()?;
            let value = parts[1].parse::<f64>()?;
            let category = parts[2].to_string();
            
            if !self.validate_record(&category, value) {
                return Err(format!("Validation failed at line {}", line_num + 1).into());
            }
            
            self.records.push(DataRecord { id, value, category });
            count += 1;
        }
        
        Ok(count)
    }

    pub fn calculate_statistics(&self) -> (f64, f64, f64) {
        if self.records.is_empty() {
            return (0.0, 0.0, 0.0);
        }
        
        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        let count = self.records.len() as f64;
        let mean = sum / count;
        
        let variance: f64 = self.records
            .iter()
            .map(|r| (r.value - mean).powi(2))
            .sum::<f64>() / count;
        
        let std_dev = variance.sqrt();
        
        (mean, variance, std_dev)
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    pub fn get_records(&self) -> &[DataRecord] {
        &self.records
    }

    fn validate_record(&self, category: &str, value: f64) -> bool {
        !category.is_empty() && value >= 0.0 && value <= 1000.0
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
        writeln!(temp_file, "id,value,category").unwrap();
        writeln!(temp_file, "1,100.5,TypeA").unwrap();
        writeln!(temp_file, "2,200.3,TypeB").unwrap();
        writeln!(temp_file, "3,150.7,TypeA").unwrap();
        
        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 3);
        
        let stats = processor.calculate_statistics();
        assert!(stats.0 > 0.0);
        
        let filtered = processor.filter_by_category("TypeA");
        assert_eq!(filtered.len(), 2);
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
    pub timestamp: String,
}

impl DataRecord {
    pub fn new(id: u32, name: String, value: f64, timestamp: String) -> Self {
        DataRecord {
            id,
            name,
            value,
            timestamp,
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.name.is_empty() && self.value >= 0.0 && !self.timestamp.is_empty()
    }
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

            let name = parts[1].to_string();
            let value = match parts[2].parse::<f64>() {
                Ok(val) => val,
                Err(_) => continue,
            };

            let timestamp = parts[3].to_string();

            let record = DataRecord::new(id, name, value, timestamp);
            if record.is_valid() {
                self.records.push(record);
                count += 1;
            }
        }

        Ok(count)
    }

    pub fn get_records(&self) -> &[DataRecord] {
        &self.records
    }

    pub fn filter_by_min_value(&self, min_value: f64) -> Vec<DataRecord> {
        self.records
            .iter()
            .filter(|record| record.value >= min_value)
            .cloned()
            .collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }

        let sum: f64 = self.records.iter().map(|record| record.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn find_by_name(&self, name: &str) -> Option<&DataRecord> {
        self.records.iter().find(|record| record.name == name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_record_validation() {
        let valid_record = DataRecord::new(1, "test".to_string(), 10.5, "2024-01-01".to_string());
        assert!(valid_record.is_valid());

        let invalid_record = DataRecord::new(2, "".to_string(), -5.0, "".to_string());
        assert!(!invalid_record.is_valid());
    }

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,timestamp").unwrap();
        writeln!(temp_file, "1,item1,10.5,2024-01-01").unwrap();
        writeln!(temp_file, "2,item2,20.0,2024-01-02").unwrap();
        writeln!(temp_file, "3,item3,5.0,2024-01-03").unwrap();

        let count = processor.load_from_csv(temp_file.path()).unwrap();
        assert_eq!(count, 3);
        assert_eq!(processor.get_records().len(), 3);

        let filtered = processor.filter_by_min_value(10.0);
        assert_eq!(filtered.len(), 2);

        let average = processor.calculate_average().unwrap();
        assert!((average - 11.83333).abs() < 0.0001);

        let found = processor.find_by_name("item2");
        assert!(found.is_some());
        assert_eq!(found.unwrap().value, 20.0);
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

        for (index, line) in reader.lines().enumerate() {
            let line = line?;
            
            if index == 0 && self.has_header {
                continue;
            }

            let fields: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            if !fields.is_empty() && !fields.iter().all(|f| f.is_empty()) {
                records.push(fields);
            }
        }

        Ok(records)
    }

    pub fn validate_records(&self, records: &[Vec<String>]) -> Vec<usize> {
        let mut invalid_indices = Vec::new();
        
        for (index, record) in records.iter().enumerate() {
            if record.is_empty() || record.iter().any(|field| field.is_empty()) {
                invalid_indices.push(index);
            }
        }
        
        invalid_indices
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
    fn test_process_file_with_header() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,score").unwrap();
        writeln!(temp_file, "Alice,25,95.5").unwrap();
        writeln!(temp_file, "Bob,30,87.2").unwrap();
        
        let processor = DataProcessor::new(',', true);
        let result = processor.process_file(temp_file.path()).unwrap();
        
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["Alice", "25", "95.5"]);
    }

    #[test]
    fn test_validate_records() {
        let records = vec![
            vec!["a".to_string(), "b".to_string()],
            vec!["".to_string(), "c".to_string()],
            vec!["d".to_string()],
            vec![],
        ];
        
        let processor = DataProcessor::new(',', false);
        let invalid = processor.validate_records(&records);
        
        assert_eq!(invalid, vec![1, 3]);
    }

    #[test]
    fn test_calculate_statistics() {
        let records = vec![
            vec!["10.5".to_string()],
            vec!["20.3".to_string()],
            vec!["15.7".to_string()],
        ];
        
        let processor = DataProcessor::new(',', false);
        let stats = processor.calculate_statistics(&records, 0).unwrap();
        
        assert!((stats.0 - 15.5).abs() < 0.01);
        assert!((stats.2 - 4.08).abs() < 0.01);
    }
}
use csv;
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

impl Record {
    fn is_valid(&self) -> bool {
        !self.name.is_empty() && self.value >= 0.0
    }
    
    fn process(&mut self) {
        self.name = self.name.to_uppercase();
        self.value = (self.value * 100.0).round() / 100.0;
    }
}

pub fn load_and_process_csv(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let file = File::open(input_path)?;
    let mut rdr = csv::Reader::from_reader(file);
    
    let mut records: Vec<Record> = Vec::new();
    
    for result in rdr.deserialize() {
        let mut record: Record = result?;
        
        if record.is_valid() {
            record.process();
            records.push(record);
        }
    }
    
    let output_file = File::create(output_path)?;
    let mut wtr = csv::Writer::from_writer(output_file);
    
    for record in records {
        wtr.serialize(record)?;
    }
    
    wtr.flush()?;
    
    println!("Processed {} valid records", records.len());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_record_validation() {
        let valid_record = Record {
            id: 1,
            name: "test".to_string(),
            value: 10.5,
            active: true,
        };
        
        let invalid_record = Record {
            id: 2,
            name: "".to_string(),
            value: -5.0,
            active: false,
        };
        
        assert!(valid_record.is_valid());
        assert!(!invalid_record.is_valid());
    }
    
    #[test]
    fn test_record_processing() {
        let mut record = Record {
            id: 1,
            name: "hello".to_string(),
            value: 12.3456,
            active: true,
        };
        
        record.process();
        
        assert_eq!(record.name, "HELLO");
        assert_eq!(record.value, 12.35);
    }
}