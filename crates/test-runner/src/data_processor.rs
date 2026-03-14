use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct DataRecord {
    id: u32,
    value: f64,
    category: String,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: String) -> Self {
        DataRecord { id, value, category }
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
        DataProcessor { records: Vec::new() }
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

            let record = DataRecord::new(id, value, category);
            if record.is_valid() {
                self.records.push(record);
                count += 1;
            }
        }

        Ok(count)
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
        let valid_record = DataRecord::new(1, 10.5, "A".to_string());
        assert!(valid_record.is_valid());

        let invalid_record = DataRecord::new(0, -5.0, "".to_string());
        assert!(!invalid_record.is_valid());
    }

    #[test]
    fn test_data_processor() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category").unwrap();
        writeln!(temp_file, "1,10.5,TypeA").unwrap();
        writeln!(temp_file, "2,15.3,TypeB").unwrap();
        writeln!(temp_file, "3,20.1,TypeA").unwrap();

        let mut processor = DataProcessor::new();
        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(processor.count_records(), 3);

        let average = processor.calculate_average();
        assert!(average.is_some());
        assert!((average.unwrap() - 15.3).abs() < 0.01);

        let filtered = processor.filter_by_category("TypeA");
        assert_eq!(filtered.len(), 2);
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
            
            if !fields.is_empty() && !fields.iter().all(|f| f.is_empty()) {
                records.push(fields);
            }
        }

        Ok(records)
    }

    pub fn validate_records(&self, records: &[Vec<String>]) -> Vec<usize> {
        let mut invalid_indices = Vec::new();
        
        for (index, record) in records.iter().enumerate() {
            if record.iter().any(|field| field.is_empty()) {
                invalid_indices.push(index);
            }
        }
        
        invalid_indices
    }

    pub fn calculate_statistics(&self, records: &[Vec<String>], column_index: usize) -> Option<(f64, f64, f64)> {
        let mut values = Vec::new();
        
        for record in records {
            if column_index < record.len() {
                if let Ok(num) = record[column_index].parse::<f64>() {
                    values.push(num);
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
    fn test_process_file_with_header() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "John,25,New York").unwrap();
        writeln!(temp_file, "Alice,30,London").unwrap();
        
        let processor = DataProcessor::new(',', true);
        let result = processor.process_file(temp_file.path()).unwrap();
        
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["John", "25", "New York"]);
    }

    #[test]
    fn test_validate_records() {
        let records = vec![
            vec!["a".to_string(), "b".to_string(), "c".to_string()],
            vec!["x".to_string(), "".to_string(), "z".to_string()],
            vec!["".to_string(), "".to_string(), "".to_string()],
        ];
        
        let processor = DataProcessor::new(',', false);
        let invalid = processor.validate_records(&records);
        
        assert_eq!(invalid, vec![1, 2]);
    }

    #[test]
    fn test_calculate_statistics() {
        let records = vec![
            vec!["10.5".to_string()],
            vec!["20.0".to_string()],
            vec!["15.5".to_string()],
        ];
        
        let processor = DataProcessor::new(',', false);
        let stats = processor.calculate_statistics(&records, 0).unwrap();
        
        assert!((stats.0 - 15.333).abs() < 0.001);
        assert!((stats.1 - 16.222).abs() < 0.001);
        assert!((stats.2 - 4.027).abs() < 0.001);
    }
}use std::collections::HashMap;

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

    pub fn validate(&self) -> Result<(), String> {
        if self.id == 0 {
            return Err("ID cannot be zero".to_string());
        }
        
        if self.values.is_empty() {
            return Err("Values cannot be empty".to_string());
        }

        for value in &self.values {
            if !value.is_finite() {
                return Err("Values must be finite numbers".to_string());
            }
        }

        Ok(())
    }

    pub fn transform(&mut self, factor: f64) {
        for value in &mut self.values {
            *value *= factor;
        }
    }

    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }
}

pub fn process_records(records: &mut [DataRecord], factor: f64) -> Result<(), String> {
    for record in records.iter_mut() {
        record.validate()?;
        record.transform(factor);
    }
    Ok(())
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
    }

    #[test]
    fn test_record_transformation() {
        let mut record = DataRecord::new(1, vec![1.0, 2.0, 3.0]);
        record.transform(2.0);
        assert_eq!(record.values, vec![2.0, 4.0, 6.0]);
    }

    #[test]
    fn test_metadata_operations() {
        let mut record = DataRecord::new(1, vec![1.0]);
        record.add_metadata("source".to_string(), "test".to_string());
        assert_eq!(record.metadata.get("source"), Some(&"test".to_string()));
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct DataRecord {
    id: u32,
    value: f64,
    category: String,
    valid: bool,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: &str) -> Self {
        let valid = value >= 0.0 && value <= 1000.0;
        DataRecord {
            id,
            value,
            category: category.to_string(),
            valid,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.valid
    }

    pub fn get_value(&self) -> f64 {
        self.value
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

    pub fn add_record(&mut self, record: DataRecord) {
        self.records.push(record);
    }

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        let mut content = String::new();
        reader.read_to_string(&mut content)?;

        for (line_num, line) in content.lines().enumerate().skip(1) {
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() >= 3 {
                let id = parts[0].parse::<u32>().unwrap_or_default();
                let value = parts[1].parse::<f64>().unwrap_or_default();
                let category = parts[2];
                
                let record = DataRecord::new(id, value, category);
                self.add_record(record);
            }
        }

        Ok(())
    }

    pub fn filter_valid(&self) -> Vec<&DataRecord> {
        self.records.iter().filter(|r| r.is_valid()).collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        let valid_records: Vec<&DataRecord> = self.filter_valid();
        if valid_records.is_empty() {
            return None;
        }

        let sum: f64 = valid_records.iter().map(|r| r.get_value()).sum();
        Some(sum / valid_records.len() as f64)
    }

    pub fn count_by_category(&self) -> std::collections::HashMap<String, usize> {
        let mut categories = std::collections::HashMap::new();
        
        for record in &self.records {
            if record.is_valid() {
                *categories.entry(record.category.clone()).or_insert(0) += 1;
            }
        }
        
        categories
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_record_validation() {
        let valid_record = DataRecord::new(1, 500.0, "A");
        assert!(valid_record.is_valid());

        let invalid_record = DataRecord::new(2, -10.0, "B");
        assert!(!invalid_record.is_valid());
    }

    #[test]
    fn test_data_processor_average() {
        let mut processor = DataProcessor::new();
        processor.add_record(DataRecord::new(1, 100.0, "Test"));
        processor.add_record(DataRecord::new(2, 200.0, "Test"));
        processor.add_record(DataRecord::new(3, 300.0, "Test"));

        assert_eq!(processor.calculate_average(), Some(200.0));
    }

    #[test]
    fn test_csv_loading() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category").unwrap();
        writeln!(temp_file, "1,100.5,CategoryA").unwrap();
        writeln!(temp_file, "2,200.3,CategoryB").unwrap();
        
        let mut processor = DataProcessor::new();
        let result = processor.load_from_csv(temp_file.path());
        
        assert!(result.is_ok());
        assert_eq!(processor.records.len(), 2);
    }
}use csv::{Reader, Writer};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
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

    fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let mut rdr = Reader::from_reader(file);
        
        for result in rdr.deserialize() {
            let record: Record = result?;
            self.records.push(record);
        }
        
        Ok(())
    }

    fn filter_by_category(&self, category: &str) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    fn calculate_average(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }
        
        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        sum / self.records.len() as f64
    }

    fn save_filtered_to_csv(&self, category: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
        let filtered = self.filter_by_category(category);
        
        let file = File::create(output_path)?;
        let mut wtr = Writer::from_writer(file);
        
        for record in filtered {
            wtr.serialize(record)?;
        }
        
        wtr.flush()?;
        Ok(())
    }

    fn find_max_value(&self) -> Option<&Record> {
        self.records.iter().max_by(|a, b| {
            a.value.partial_cmp(&b.value).unwrap()
        })
    }
}

fn process_data() -> Result<(), Box<dyn Error>> {
    let mut processor = DataProcessor::new();
    
    processor.load_from_csv("input_data.csv")?;
    
    println!("Total records loaded: {}", processor.records.len());
    println!("Average value: {:.2}", processor.calculate_average());
    
    if let Some(max_record) = processor.find_max_value() {
        println!("Record with maximum value: {:?}", max_record);
    }
    
    let filtered = processor.filter_by_category("premium");
    println!("Premium records found: {}", filtered.len());
    
    processor.save_filtered_to_csv("premium", "premium_records.csv")?;
    
    Ok(())
}

fn main() {
    if let Err(e) = process_data() {
        eprintln!("Error processing data: {}", e);
        std::process::exit(1);
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
            return Err("Empty data set provided".to_string());
        }

        if let Some(cached) = self.cache.get(key) {
            return Ok(cached.clone());
        }

        let validated = self.validate_data(values)?;
        let normalized = self.normalize_data(&validated);
        let transformed = self.apply_transformations(&normalized);

        self.cache.insert(key.to_string(), transformed.clone());
        Ok(transformed)
    }

    fn validate_data(&self, data: &[f64]) -> Result<Vec<f64>, String> {
        for &value in data {
            if !value.is_finite() {
                return Err("Invalid numeric value detected".to_string());
            }
        }
        Ok(data.to_vec())
    }

    fn normalize_data(&self, data: &[f64]) -> Vec<f64> {
        let mean = data.iter().sum::<f64>() / data.len() as f64;
        let variance = data.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / data.len() as f64;
        
        if variance.abs() < 1e-10 {
            return vec![0.0; data.len()];
        }

        data.iter()
            .map(|&x| (x - mean) / variance.sqrt())
            .collect()
    }

    fn apply_transformations(&self, data: &[f64]) -> Vec<f64> {
        data.iter()
            .map(|&x| x.powi(2).ln_1p())
            .collect()
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    pub fn get_cache_stats(&self) -> (usize, usize) {
        let total_items = self.cache.len();
        let total_values = self.cache.values()
            .map(|v| v.len())
            .sum();
        (total_items, total_values)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_processing() {
        let mut processor = DataProcessor::new();
        let test_data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        
        let result = processor.process_numeric_data("test", &test_data);
        assert!(result.is_ok());
        
        let processed = result.unwrap();
        assert_eq!(processed.len(), test_data.len());
    }

    #[test]
    fn test_invalid_data() {
        let mut processor = DataProcessor::new();
        let invalid_data = vec![1.0, f64::NAN, 3.0];
        
        let result = processor.process_numeric_data("invalid", &invalid_data);
        assert!(result.is_err());
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
    active: bool,
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

pub fn calculate_statistics(records: &[Record]) -> (f64, f64, usize) {
    let sum: f64 = records.iter().map(|r| r.value).sum();
    let count = records.len();
    let average = if count > 0 { sum / count as f64 } else { 0.0 };
    
    let active_count = records.iter().filter(|r| r.active).count();
    
    (sum, average, active_count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_process_valid_csv() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "id,name,value,active").unwrap();
        writeln!(file, "1,Test1,10.5,true").unwrap();
        writeln!(file, "2,Test2,-5.0,false").unwrap();
        writeln!(file, "3,Test3,20.0,true").unwrap();
        
        let records = process_csv_data(file.path().to_str().unwrap()).unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].name, "Test1");
        assert_eq!(records[1].value, 20.0);
    }

    #[test]
    fn test_calculate_statistics() {
        let records = vec![
            Record { id: 1, name: "A".to_string(), value: 10.0, active: true },
            Record { id: 2, name: "B".to_string(), value: 20.0, active: false },
            Record { id: 3, name: "C".to_string(), value: 30.0, active: true },
        ];
        
        let (sum, avg, active_count) = calculate_statistics(&records);
        assert_eq!(sum, 60.0);
        assert_eq!(avg, 20.0);
        assert_eq!(active_count, 2);
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;

pub struct DataProcessor {
    data: Vec<f64>,
    frequency_map: HashMap<String, usize>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            data: Vec::new(),
            frequency_map: HashMap::new(),
        }
    }

    pub fn load_numeric_data(&mut self, filepath: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(filepath)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            if let Ok(value) = line.trim().parse::<f64>() {
                self.data.push(value);
            }
        }
        Ok(())
    }

    pub fn load_categorical_data(&mut self, filepath: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(filepath)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            let category = line.trim().to_string();
            *self.frequency_map.entry(category).or_insert(0) += 1;
        }
        Ok(())
    }

    pub fn calculate_statistics(&self) -> (f64, f64, f64, f64) {
        if self.data.is_empty() {
            return (0.0, 0.0, 0.0, 0.0);
        }

        let sum: f64 = self.data.iter().sum();
        let count = self.data.len() as f64;
        let mean = sum / count;

        let variance: f64 = self.data.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / count;

        let std_dev = variance.sqrt();

        let mut sorted_data = self.data.clone();
        sorted_data.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let median = if count as usize % 2 == 0 {
            let mid = count as usize / 2;
            (sorted_data[mid - 1] + sorted_data[mid]) / 2.0
        } else {
            sorted_data[count as usize / 2]
        };

        (mean, median, variance, std_dev)
    }

    pub fn get_frequency_distribution(&self) -> Vec<(&String, &usize)> {
        let mut entries: Vec<_> = self.frequency_map.iter().collect();
        entries.sort_by(|a, b| b.1.cmp(a.1));
        entries
    }

    pub fn filter_data(&self, threshold: f64) -> Vec<f64> {
        self.data.iter()
            .filter(|&&x| x > threshold)
            .cloned()
            .collect()
    }

    pub fn normalize_data(&mut self) {
        if self.data.is_empty() {
            return;
        }

        let (mean, _, _, std_dev) = self.calculate_statistics();
        
        if std_dev > 0.0 {
            for value in &mut self.data {
                *value = (*value - mean) / std_dev;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_numeric_data_processing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "10.5\n20.3\n15.7\n25.1\n18.9").unwrap();
        
        let mut processor = DataProcessor::new();
        processor.load_numeric_data(temp_file.path().to_str().unwrap()).unwrap();
        
        let (mean, median, variance, std_dev) = processor.calculate_statistics();
        
        assert!((mean - 18.1).abs() < 0.1);
        assert!((median - 18.9).abs() < 0.1);
        assert!(variance > 0.0);
        assert!(std_dev > 0.0);
    }

    #[test]
    fn test_categorical_data_processing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "apple\nbanana\napple\norange\nbanana\nbanana").unwrap();
        
        let mut processor = DataProcessor::new();
        processor.load_categorical_data(temp_file.path().to_str().unwrap()).unwrap();
        
        let distribution = processor.get_frequency_distribution();
        
        assert_eq!(distribution[0].0, "banana");
        assert_eq!(*distribution[0].1, 3);
        assert_eq!(distribution[1].0, "apple");
        assert_eq!(*distribution[1].1, 2);
    }

    #[test]
    fn test_data_filtering() {
        let mut processor = DataProcessor::new();
        processor.data = vec![5.0, 15.0, 10.0, 20.0, 25.0];
        
        let filtered = processor.filter_data(12.0);
        
        assert_eq!(filtered.len(), 3);
        assert!(filtered.contains(&15.0));
        assert!(filtered.contains(&20.0));
        assert!(filtered.contains(&25.0));
    }
}
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DataRecord {
    pub id: u32,
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

impl std::fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
}

#[derive(Debug, Clone)]
pub struct ProcessorConfig {
    pub max_values: usize,
    pub require_timestamp: bool,
    pub allowed_metadata_keys: Vec<String>,
}

impl DataProcessor {
    pub fn new(config: ProcessorConfig) -> Self {
        DataProcessor { config }
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if record.values.len() > self.config.max_values {
            return Err(ProcessingError::ValidationError(
                format!("Too many values: {} > {}", record.values.len(), self.config.max_values)
            ));
        }

        if self.config.require_timestamp && record.timestamp <= 0 {
            return Err(ProcessingError::ValidationError(
                "Invalid timestamp".to_string()
            ));
        }

        for key in record.metadata.keys() {
            if !self.config.allowed_metadata_keys.contains(key) {
                return Err(ProcessingError::ValidationError(
                    format!("Disallowed metadata key: {}", key)
                ));
            }
        }

        Ok(())
    }

    pub fn transform_record(&self, record: &DataRecord) -> Result<DataRecord, ProcessingError> {
        let mut transformed = record.clone();
        
        transformed.values = record.values
            .iter()
            .map(|&v| {
                if v.is_nan() || v.is_infinite() {
                    0.0
                } else {
                    v
                }
            })
            .collect();

        transformed.metadata.retain(|k, _| 
            self.config.allowed_metadata_keys.contains(k)
        );

        if transformed.values.is_empty() {
            return Err(ProcessingError::TransformationFailed(
                "No valid values after transformation".to_string()
            ));
        }

        Ok(transformed)
    }

    pub fn process_batch(&self, records: Vec<DataRecord>) -> Result<Vec<DataRecord>, ProcessingError> {
        let mut processed = Vec::with_capacity(records.len());
        
        for record in records {
            self.validate_record(&record)?;
            let transformed = self.transform_record(&record)?;
            processed.push(transformed);
        }

        Ok(processed)
    }

    pub fn calculate_statistics(&self, records: &[DataRecord]) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        
        if records.is_empty() {
            return stats;
        }

        let total_values: Vec<f64> = records
            .iter()
            .flat_map(|r| r.values.iter().copied())
            .collect();

        if !total_values.is_empty() {
            let sum: f64 = total_values.iter().sum();
            let count = total_values.len() as f64;
            let mean = sum / count;
            
            let variance: f64 = total_values
                .iter()
                .map(|&v| (v - mean).powi(2))
                .sum::<f64>() / count;

            stats.insert("mean".to_string(), mean);
            stats.insert("variance".to_string(), variance);
            stats.insert("count".to_string(), count);
            stats.insert("sum".to_string(), sum);
        }

        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> ProcessorConfig {
        ProcessorConfig {
            max_values: 10,
            require_timestamp: true,
            allowed_metadata_keys: vec!["source".to_string(), "version".to_string()],
        }
    }

    fn create_valid_record() -> DataRecord {
        DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: vec![1.0, 2.0, 3.0],
            metadata: {
                let mut map = HashMap::new();
                map.insert("source".to_string(), "test".to_string());
                map
            },
        }
    }

    #[test]
    fn test_validation_success() {
        let processor = DataProcessor::new(create_test_config());
        let record = create_valid_record();
        
        assert!(processor.validate_record(&record).is_ok());
    }

    #[test]
    fn test_validation_too_many_values() {
        let processor = DataProcessor::new(create_test_config());
        let mut record = create_valid_record();
        record.values = vec![0.0; 15];
        
        assert!(processor.validate_record(&record).is_err());
    }

    #[test]
    fn test_transformation() {
        let processor = DataProcessor::new(create_test_config());
        let mut record = create_valid_record();
        record.values = vec![1.0, f64::NAN, 3.0];
        
        let result = processor.transform_record(&record);
        assert!(result.is_ok());
        let transformed = result.unwrap();
        assert_eq!(transformed.values, vec![1.0, 0.0, 3.0]);
    }

    #[test]
    fn test_process_batch() {
        let processor = DataProcessor::new(create_test_config());
        let records = vec![create_valid_record(), create_valid_record()];
        
        let result = processor.process_batch(records);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 2);
    }

    #[test]
    fn test_statistics_calculation() {
        let processor = DataProcessor::new(create_test_config());
        let records = vec![
            DataRecord {
                id: 1,
                timestamp: 1234567890,
                values: vec![1.0, 2.0],
                metadata: HashMap::new(),
            },
            DataRecord {
                id: 2,
                timestamp: 1234567891,
                values: vec![3.0, 4.0],
                metadata: HashMap::new(),
            },
        ];
        
        let stats = processor.calculate_statistics(&records);
        assert_eq!(stats.get("mean"), Some(&2.5));
        assert_eq!(stats.get("count"), Some(&4.0));
    }
}use std::error::Error;
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

    pub fn calculate_statistics(&self, column_name: &str) -> Option<(f64, f64, f64)> {
        let mut values = Vec::new();
        
        for record in &self.records {
            if let Some(value_str) = record.get(column_name) {
                if let Ok(value) = value_str.parse::<f64>() {
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
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / count;
        
        let std_dev = variance.sqrt();
        
        Some((mean, variance, std_dev))
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processor() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,score").unwrap();
        writeln!(temp_file, "Alice,25,85.5").unwrap();
        writeln!(temp_file, "Bob,30,92.0").unwrap();
        writeln!(temp_file, "Charlie,25,78.5").unwrap();
        
        let file_path = temp_file.path().to_str().unwrap();
        
        let mut processor = DataProcessor::new();
        let result = processor.load_csv(file_path);
        assert!(result.is_ok());
        
        let stats = processor.calculate_statistics("score");
        assert!(stats.is_some());
        
        let (mean, _, std_dev) = stats.unwrap();
        assert!((mean - 85.333).abs() < 0.001);
        assert!(std_dev > 0.0);
        
        let filtered = processor.filter_records(|record| {
            record.get("age").map(|age| age == "25").unwrap_or(false)
        });
        assert_eq!(filtered.len(), 2);
        
        let unique_ages = processor.get_unique_values("age");
        assert_eq!(unique_ages.len(), 2);
    }
}