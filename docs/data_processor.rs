
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
        let mut lines = reader.lines().enumerate();

        if self.has_header {
            lines.next();
        }

        for (line_number, line) in lines {
            let line = line?;
            let fields: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            if fields.iter().any(|f| f.is_empty()) {
                return Err(format!("Empty field detected at line {}", line_number + 1).into());
            }

            records.push(fields);
        }

        if records.is_empty() {
            return Err("No valid data records found".into());
        }

        Ok(records)
    }

    pub fn validate_records(&self, records: &[Vec<String>]) -> Result<(), Box<dyn Error>> {
        if records.is_empty() {
            return Err("No records to validate".into());
        }

        let expected_len = records[0].len();
        
        for (idx, record) in records.iter().enumerate() {
            if record.len() != expected_len {
                return Err(format!("Record {} has {} fields, expected {}", 
                    idx + 1, record.len(), expected_len).into());
            }

            for (field_idx, field) in record.iter().enumerate() {
                if field.trim().is_empty() {
                    return Err(format!("Empty field at record {}, position {}", 
                        idx + 1, field_idx + 1).into());
                }
            }
        }

        Ok(())
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
        writeln!(temp_file, "John,30,New York").unwrap();
        writeln!(temp_file, "Alice,25,London").unwrap();

        let processor = DataProcessor::new(',', true);
        let result = processor.process_file(temp_file.path()).unwrap();
        
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["John", "30", "New York"]);
    }

    #[test]
    fn test_validate_records() {
        let processor = DataProcessor::new(',', false);
        let valid_records = vec![
            vec!["a".to_string(), "b".to_string()],
            vec!["c".to_string(), "d".to_string()],
        ];
        
        assert!(processor.validate_records(&valid_records).is_ok());
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

    pub fn process_numeric_data(&mut self, key: &str, data: &[f64]) -> Result<Vec<f64>, String> {
        if data.is_empty() {
            return Err("Empty data provided".to_string());
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
            if value.is_nan() || value.is_infinite() {
                return Err("Invalid numeric value detected".to_string());
            }
            if value < 0.0 {
                return Err("Negative values not allowed".to_string());
            }
            result.push(value);
        }
        
        Ok(result)
    }

    fn normalize_data(&self, data: &[f64]) -> Vec<f64> {
        let max_value = data.iter().fold(f64::MIN, |a, &b| a.max(b));
        
        if max_value == 0.0 {
            return vec![0.0; data.len()];
        }

        data.iter()
            .map(|&x| x / max_value)
            .collect()
    }

    fn apply_transformations(&self, data: &[f64]) -> Vec<f64> {
        data.iter()
            .map(|&x| (x * 100.0).round() / 100.0)
            .collect()
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
    fn test_validation_rejects_negative() {
        let processor = DataProcessor::new();
        let data = vec![1.0, -2.0, 3.0];
        assert!(processor.validate_data(&data).is_err());
    }

    #[test]
    fn test_normalization_scales_correctly() {
        let processor = DataProcessor::new();
        let data = vec![1.0, 2.0, 3.0];
        let normalized = processor.normalize_data(&data);
        assert_eq!(normalized, vec![1.0/3.0, 2.0/3.0, 1.0]);
    }

    #[test]
    fn test_cache_functionality() {
        let mut processor = DataProcessor::new();
        let data = vec![1.0, 2.0, 3.0];
        
        let result1 = processor.process_numeric_data("test", &data).unwrap();
        let result2 = processor.process_numeric_data("test", &data).unwrap();
        
        assert_eq!(result1, result2);
        assert_eq!(processor.cache_size(), 1);
    }
}
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct ValidationError {
    message: String,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Validation error: {}", self.message)
    }
}

impl Error for ValidationError {}

pub struct DataProcessor {
    threshold: f64,
}

impl DataProcessor {
    pub fn new(threshold: f64) -> Result<Self, ValidationError> {
        if threshold < 0.0 || threshold > 1.0 {
            return Err(ValidationError {
                message: format!("Threshold must be between 0.0 and 1.0, got {}", threshold),
            });
        }
        
        Ok(Self { threshold })
    }
    
    pub fn process_data(&self, input: &[f64]) -> Result<Vec<f64>, ValidationError> {
        if input.is_empty() {
            return Err(ValidationError {
                message: "Input data cannot be empty".to_string(),
            });
        }
        
        let normalized: Vec<f64> = input.iter()
            .map(|&value| {
                if value.is_nan() || value.is_infinite() {
                    0.0
                } else {
                    value
                }
            })
            .collect();
        
        let max_value = normalized.iter()
            .fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        
        if max_value <= 0.0 {
            return Err(ValidationError {
                message: "All values must be positive for processing".to_string(),
            });
        }
        
        let scaled: Vec<f64> = normalized.iter()
            .map(|&value| value / max_value)
            .collect();
        
        let filtered: Vec<f64> = scaled.iter()
            .filter(|&&value| value >= self.threshold)
            .cloned()
            .collect();
        
        Ok(filtered)
    }
    
    pub fn calculate_statistics(&self, data: &[f64]) -> Result<(f64, f64, f64), ValidationError> {
        if data.len() < 2 {
            return Err(ValidationError {
                message: "Insufficient data for statistics calculation".to_string(),
            });
        }
        
        let sum: f64 = data.iter().sum();
        let mean = sum / data.len() as f64;
        
        let variance: f64 = data.iter()
            .map(|&value| {
                let diff = value - mean;
                diff * diff
            })
            .sum::<f64>() / data.len() as f64;
        
        let std_dev = variance.sqrt();
        
        Ok((mean, variance, std_dev))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_valid_processor_creation() {
        let processor = DataProcessor::new(0.5);
        assert!(processor.is_ok());
    }
    
    #[test]
    fn test_invalid_threshold() {
        let processor = DataProcessor::new(1.5);
        assert!(processor.is_err());
    }
    
    #[test]
    fn test_data_processing() {
        let processor = DataProcessor::new(0.3).unwrap();
        let input = vec![1.0, 2.0, 3.0, 0.1, 0.5];
        let result = processor.process_data(&input);
        assert!(result.is_ok());
        
        let processed = result.unwrap();
        assert_eq!(processed.len(), 3);
    }
    
    #[test]
    fn test_statistics_calculation() {
        let processor = DataProcessor::new(0.5).unwrap();
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let stats = processor.calculate_statistics(&data);
        assert!(stats.is_ok());
        
        let (mean, variance, std_dev) = stats.unwrap();
        assert!((mean - 3.0).abs() < 1e-10);
        assert!((variance - 2.0).abs() < 1e-10);
        assert!((std_dev - 2.0_f64.sqrt()).abs() < 1e-10);
    }
}
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct DataRecord {
    id: u64,
    values: Vec<f64>,
    metadata: HashMap<String, String>,
}

impl DataRecord {
    pub fn new(id: u64, values: Vec<f64>) -> Self {
        Self {
            id,
            values,
            metadata: HashMap::new(),
        }
    }

    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.id == 0 {
            return Err("Invalid record ID".to_string());
        }

        if self.values.is_empty() {
            return Err("Empty values array".to_string());
        }

        for value in &self.values {
            if value.is_nan() || value.is_infinite() {
                return Err("Invalid numeric value detected".to_string());
            }
        }

        Ok(())
    }

    pub fn normalize(&mut self) {
        if self.values.is_empty() {
            return;
        }

        let sum: f64 = self.values.iter().sum();
        if sum != 0.0 {
            for value in self.values.iter_mut() {
                *value /= sum;
            }
        }
    }

    pub fn calculate_statistics(&self) -> HashMap<String, f64> {
        let mut stats = HashMap::new();

        if self.values.is_empty() {
            return stats;
        }

        let count = self.values.len() as f64;
        let sum: f64 = self.values.iter().sum();
        let mean = sum / count;

        let variance: f64 = self.values
            .iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / count;

        stats.insert("count".to_string(), count);
        stats.insert("sum".to_string(), sum);
        stats.insert("mean".to_string(), mean);
        stats.insert("variance".to_string(), variance);

        stats
    }
}

pub fn process_records(records: &mut [DataRecord]) -> Vec<Result<HashMap<String, f64>, String>> {
    records
        .iter_mut()
        .map(|record| {
            record.validate()?;
            record.normalize();
            Ok(record.calculate_statistics())
        })
        .collect()
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
    fn test_normalization() {
        let mut record = DataRecord::new(1, vec![1.0, 2.0, 3.0]);
        record.normalize();

        let sum: f64 = record.values.iter().sum();
        assert!((sum - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_statistics_calculation() {
        let record = DataRecord::new(1, vec![1.0, 2.0, 3.0, 4.0]);
        let stats = record.calculate_statistics();

        assert_eq!(stats.get("count"), Some(&4.0));
        assert_eq!(stats.get("sum"), Some(&10.0));
        assert_eq!(stats.get("mean"), Some(&2.5));
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

pub fn process_data_file(file_path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let mut rdr = Reader::from_reader(file);
    let mut records = Vec::new();

    for result in rdr.deserialize() {
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
        return Err("Value cannot be negative".into());
    }
    if !["A", "B", "C"].contains(&record.category.as_str()) {
        return Err("Invalid category".into());
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
    let variance: f64 = records.iter()
        .map(|r| (r.value - mean).powi(2))
        .sum::<f64>() / count;
    let std_dev = variance.sqrt();

    (mean, variance, std_dev)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_process_valid_data() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,category").unwrap();
        writeln!(temp_file, "1,Test1,10.5,A").unwrap();
        writeln!(temp_file, "2,Test2,20.0,B").unwrap();

        let result = process_data_file(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 2);
    }

    #[test]
    fn test_calculate_statistics() {
        let records = vec![
            Record { id: 1, name: "A".to_string(), value: 10.0, category: "A".to_string() },
            Record { id: 2, name: "B".to_string(), value: 20.0, category: "B".to_string() },
            Record { id: 3, name: "C".to_string(), value: 30.0, category: "C".to_string() },
        ];

        let (mean, variance, std_dev) = calculate_statistics(&records);
        assert_eq!(mean, 20.0);
        assert_eq!(variance, 66.66666666666667);
        assert_eq!(std_dev, 8.16496580927726);
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
    active: bool,
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
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_process_csv() {
        let csv_data = "id,name,value,active\n1,ItemA,10.5,true\n2,ItemB,-3.2,false\n3,ItemC,7.8,true\n";
        
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", csv_data).unwrap();
        
        let result = process_csv_file(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        
        let records = result.unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].name, "ItemA");
        assert_eq!(records[1].name, "ItemC");
    }

    #[test]
    fn test_statistics_calculation() {
        let records = vec![
            Record { id: 1, name: "Test1".to_string(), value: 10.0, active: true },
            Record { id: 2, name: "Test2".to_string(), value: 20.0, active: false },
            Record { id: 3, name: "Test3".to_string(), value: 30.0, active: true },
        ];
        
        let (sum, avg, active_count) = calculate_statistics(&records);
        assert_eq!(sum, 60.0);
        assert_eq!(avg, 20.0);
        assert_eq!(active_count, 2);
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
    pub valid: bool,
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

            let value = match parts[1].parse::<f64>() {
                Ok(val) => val,
                Err(_) => continue,
            };

            let category = parts[2].to_string();
            let valid = parts[3].parse::<bool>().unwrap_or(false);

            let record = DataRecord {
                id,
                value,
                category,
                valid,
            };

            self.records.push(record);
            count += 1;
        }

        Ok(count)
    }

    pub fn filter_valid(&self) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.valid)
            .collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        let valid_records: Vec<&DataRecord> = self.records
            .iter()
            .filter(|r| r.valid)
            .collect();

        if valid_records.is_empty() {
            return None;
        }

        let sum: f64 = valid_records.iter().map(|r| r.value).sum();
        Some(sum / valid_records.len() as f64)
    }

    pub fn group_by_category(&self) -> std::collections::HashMap<String, Vec<&DataRecord>> {
        let mut groups = std::collections::HashMap::new();

        for record in &self.records {
            groups
                .entry(record.category.clone())
                .or_insert_with(Vec::new)
                .push(record);
        }

        groups
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
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        assert_eq!(processor.count_records(), 0);

        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category,valid").unwrap();
        writeln!(temp_file, "1,10.5,TypeA,true").unwrap();
        writeln!(temp_file, "2,20.3,TypeB,false").unwrap();
        writeln!(temp_file, "3,15.7,TypeA,true").unwrap();

        let count = processor.load_from_csv(temp_file.path()).unwrap();
        assert_eq!(count, 3);
        assert_eq!(processor.count_records(), 3);

        let valid_records = processor.filter_valid();
        assert_eq!(valid_records.len(), 2);

        let average = processor.calculate_average();
        assert!(average.is_some());
        assert!((average.unwrap() - 13.1).abs() < 0.001);

        let groups = processor.group_by_category();
        assert_eq!(groups.get("TypeA").unwrap().len(), 2);
        assert_eq!(groups.get("TypeB").unwrap().len(), 1);
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, PartialEq)]
pub struct DataRecord {
    id: u32,
    value: f64,
    category: String,
    valid: bool,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: &str) -> Self {
        let valid = value >= 0.0 && !category.is_empty();
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

    pub fn process(&mut self) {
        if self.value < 100.0 {
            self.value *= 1.1;
        }
    }
}

pub fn load_csv_data<P: AsRef<Path>>(path: P) -> Result<Vec<DataRecord>, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();

    for (line_num, line) in reader.lines().enumerate() {
        let line = line?;
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() != 3 {
            continue;
        }

        let id = match parts[0].parse::<u32>() {
            Ok(id) => id,
            Err(_) => continue,
        };

        let value = match parts[1].parse::<f64>() {
            Ok(value) => value,
            Err(_) => continue,
        };

        let category = parts[2].trim();
        records.push(DataRecord::new(id, value, category));
    }

    Ok(records)
}

pub fn filter_valid_records(records: &[DataRecord]) -> Vec<&DataRecord> {
    records.iter().filter(|r| r.is_valid()).collect()
}

pub fn calculate_statistics(records: &[DataRecord]) -> (f64, f64, f64) {
    if records.is_empty() {
        return (0.0, 0.0, 0.0);
    }

    let sum: f64 = records.iter().map(|r| r.value).sum();
    let count = records.len() as f64;
    let mean = sum / count;

    let variance: f64 = records
        .iter()
        .map(|r| (r.value - mean).powi(2))
        .sum::<f64>()
        / count;

    let std_dev = variance.sqrt();

    (mean, variance, std_dev)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_record_creation() {
        let record = DataRecord::new(1, 42.5, "test");
        assert_eq!(record.id, 1);
        assert_eq!(record.value, 42.5);
        assert_eq!(record.category, "test");
        assert!(record.is_valid());
    }

    #[test]
    fn test_invalid_record() {
        let record = DataRecord::new(2, -5.0, "invalid");
        assert!(!record.is_valid());
    }

    #[test]
    fn test_process_record() {
        let mut record = DataRecord::new(3, 50.0, "process");
        record.process();
        assert_eq!(record.value, 55.0);
    }

    #[test]
    fn test_load_csv_data() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "1,100.5,category_a").unwrap();
        writeln!(temp_file, "2,75.3,category_b").unwrap();
        writeln!(temp_file, "# This is a comment").unwrap();
        writeln!(temp_file, "").unwrap();
        writeln!(temp_file, "3,invalid,category_c").unwrap();

        let records = load_csv_data(temp_file.path()).unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].id, 1);
        assert_eq!(records[1].category, "category_b");
    }

    #[test]
    fn test_filter_valid_records() {
        let records = vec![
            DataRecord::new(1, 10.0, "valid"),
            DataRecord::new(2, -5.0, "invalid"),
            DataRecord::new(3, 20.0, ""),
        ];

        let valid_records = filter_valid_records(&records);
        assert_eq!(valid_records.len(), 1);
        assert_eq!(valid_records[0].id, 1);
    }

    #[test]
    fn test_calculate_statistics() {
        let records = vec![
            DataRecord::new(1, 10.0, "a"),
            DataRecord::new(2, 20.0, "b"),
            DataRecord::new(3, 30.0, "c"),
        ];

        let (mean, variance, std_dev) = calculate_statistics(&records);
        assert_eq!(mean, 20.0);
        assert_eq!(variance, 66.66666666666667);
        assert_eq!(std_dev, 8.16496580927726);
    }

    #[test]
    fn test_empty_statistics() {
        let records: Vec<DataRecord> = vec![];
        let (mean, variance, std_dev) = calculate_statistics(&records);
        assert_eq!(mean, 0.0);
        assert_eq!(variance, 0.0);
        assert_eq!(std_dev, 0.0);
    }
}use csv::Reader;
use serde::Deserialize;
use std::error::Error;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

pub fn process_csv_file(file_path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let path = Path::new(file_path);
    if !path.exists() {
        return Err("File does not exist".into());
    }

    let mut reader = Reader::from_path(path)?;
    let mut records = Vec::new();

    for result in reader.deserialize() {
        let record: Record = result?;
        if record.value < 0.0 {
            return Err(format!("Invalid value in record ID {}", record.id).into());
        }
        records.push(record);
    }

    if records.is_empty() {
        return Err("No valid records found".into());
    }

    Ok(records)
}

pub fn calculate_statistics(records: &[Record]) -> (f64, f64, f64) {
    let count = records.len() as f64;
    let sum: f64 = records.iter().map(|r| r.value).sum();
    let mean = sum / count;
    
    let variance: f64 = records.iter()
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
    fn test_valid_csv_processing() {
        let csv_data = "id,name,value,category\n1,Test1,10.5,A\n2,Test2,20.3,B\n";
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", csv_data).unwrap();
        
        let result = process_csv_file(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 2);
    }

    #[test]
    fn test_invalid_negative_value() {
        let csv_data = "id,name,value,category\n1,Test1,-5.0,A\n";
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", csv_data).unwrap();
        
        let result = process_csv_file(temp_file.path().to_str().unwrap());
        assert!(result.is_err());
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

pub fn process_data_file(file_path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let mut rdr = Reader::from_reader(file);
    let mut records = Vec::new();

    for result in rdr.deserialize() {
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

pub fn filter_by_category(records: Vec<Record>, category: &str) -> Vec<Record> {
    records.into_iter()
        .filter(|r| r.category == category)
        .collect()
}use csv::Reader;
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
        
        if record.value < 0.0 {
            return Err("Negative value found in data".into());
        }
        
        if record.name.is_empty() {
            return Err("Empty name field detected".into());
        }
        
        records.push(record);
    }

    if records.is_empty() {
        return Err("No valid records found in CSV file".into());
    }

    Ok(records)
}

pub fn calculate_statistics(records: &[Record]) -> (f64, f64, f64) {
    let sum: f64 = records.iter().map(|r| r.value).sum();
    let count = records.len() as f64;
    let mean = sum / count;
    
    let variance: f64 = records.iter()
        .map(|r| (r.value - mean).powi(2))
        .sum::<f64>() / count;
    
    let std_dev = variance.sqrt();
    
    (sum, mean, std_dev)
}

pub fn filter_by_category(records: Vec<Record>, category: &str) -> Vec<Record> {
    records.into_iter()
        .filter(|r| r.category == category)
        .collect()
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
        writeln!(file, "1,ItemA,10.5,Category1").unwrap();
        writeln!(file, "2,ItemB,20.3,Category2").unwrap();
        
        let result = process_csv_data(file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 2);
    }

    #[test]
    fn test_statistics_calculation() {
        let records = vec![
            Record { id: 1, name: "Test1".to_string(), value: 10.0, category: "A".to_string() },
            Record { id: 2, name: "Test2".to_string(), value: 20.0, category: "A".to_string() },
            Record { id: 3, name: "Test3".to_string(), value: 30.0, category: "B".to_string() },
        ];
        
        let (sum, mean, std_dev) = calculate_statistics(&records);
        assert_eq!(sum, 60.0);
        assert_eq!(mean, 20.0);
        assert!(std_dev > 8.16 && std_dev < 8.17);
    }
}use std::error::Error;
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
        DataRecord {
            id,
            value,
            category,
            timestamp,
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.category.is_empty() 
            && self.value >= 0.0 
            && self.value <= 1000.0
            && self.timestamp.len() == 19
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
                Ok(id) => id,
                Err(_) => continue,
            };
            
            let value = match parts[1].parse::<f64>() {
                Ok(value) => value,
                Err(_) => continue,
            };
            
            let record = DataRecord::new(
                id,
                value,
                parts[2].to_string(),
                parts[3].to_string(),
            );
            
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
    fn test_record_validation() {
        let valid_record = DataRecord::new(
            1,
            500.0,
            "category_a".to_string(),
            "2024-01-15 10:30:00".to_string(),
        );
        assert!(valid_record.is_valid());

        let invalid_record = DataRecord::new(
            2,
            -10.0,
            "".to_string(),
            "invalid".to_string(),
        );
        assert!(!invalid_record.is_valid());
    }

    #[test]
    fn test_data_processing() {
        let mut processor = DataProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category,timestamp").unwrap();
        writeln!(temp_file, "1,100.5,category_a,2024-01-15 10:30:00").unwrap();
        writeln!(temp_file, "2,200.3,category_b,2024-01-15 11:30:00").unwrap();
        writeln!(temp_file, "3,300.7,category_a,2024-01-15 12:30:00").unwrap();
        
        let count = processor.load_from_csv(temp_file.path()).unwrap();
        assert_eq!(count, 3);
        assert_eq!(processor.count_records(), 3);
        
        let category_a = processor.filter_by_category("category_a");
        assert_eq!(category_a.len(), 2);
        
        let stats = processor.get_statistics();
        assert!(stats.0 > 99.0 && stats.0 < 101.0);
        assert!(stats.1 > 299.0 && stats.1 < 301.0);
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
    pub valid: bool,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: String) -> Self {
        let valid = value >= 0.0 && !category.is_empty();
        Self {
            id,
            value,
            category,
            valid,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.id == 0 {
            return Err("ID cannot be zero".to_string());
        }
        if self.value < 0.0 {
            return Err("Value must be non-negative".to_string());
        }
        if self.category.is_empty() {
            return Err("Category cannot be empty".to_string());
        }
        Ok(())
    }
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
    stats: ProcessingStats,
}

#[derive(Debug, Default)]
pub struct ProcessingStats {
    pub total_records: usize,
    pub valid_records: usize,
    pub average_value: f64,
    pub categories: Vec<String>,
}

impl DataProcessor {
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
            stats: ProcessingStats::default(),
        }
    }

    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        
        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            if line.trim().is_empty() || line.starts_with('#') {
                continue;
            }
            
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() != 3 {
                return Err(format!("Invalid format at line {}", line_num + 1).into());
            }
            
            let id = parts[0].parse::<u32>()?;
            let value = parts[1].parse::<f64>()?;
            let category = parts[2].trim().to_string();
            
            let record = DataRecord::new(id, value, category);
            self.records.push(record);
        }
        
        self.calculate_stats();
        Ok(())
    }

    pub fn add_record(&mut self, record: DataRecord) {
        self.records.push(record);
        self.calculate_stats();
    }

    pub fn get_valid_records(&self) -> Vec<&DataRecord> {
        self.records.iter().filter(|r| r.valid).collect()
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|r| r.category == category && r.valid)
            .collect()
    }

    pub fn calculate_stats(&mut self) {
        self.stats.total_records = self.records.len();
        self.stats.valid_records = self.records.iter().filter(|r| r.valid).count();
        
        let valid_values: Vec<f64> = self.records
            .iter()
            .filter(|r| r.valid)
            .map(|r| r.value)
            .collect();
        
        self.stats.average_value = if !valid_values.is_empty() {
            valid_values.iter().sum::<f64>() / valid_values.len() as f64
        } else {
            0.0
        };
        
        let mut categories: Vec<String> = self.records
            .iter()
            .map(|r| r.category.clone())
            .collect();
        categories.sort();
        categories.dedup();
        self.stats.categories = categories;
    }

    pub fn get_stats(&self) -> &ProcessingStats {
        &self.stats
    }

    pub fn export_valid_records(&self) -> String {
        let mut output = String::new();
        output.push_str("id,value,category,valid\n");
        
        for record in &self.records {
            if record.valid {
                output.push_str(&format!("{},{},{},{}\n", 
                    record.id, 
                    record.value, 
                    record.category, 
                    record.valid));
            }
        }
        
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_record_validation() {
        let valid_record = DataRecord::new(1, 10.5, "test".to_string());
        assert!(valid_record.validate().is_ok());
        assert!(valid_record.valid);

        let invalid_record = DataRecord::new(0, -5.0, "".to_string());
        assert!(invalid_record.validate().is_err());
        assert!(!invalid_record.valid);
    }

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        
        let record1 = DataRecord::new(1, 10.0, "A".to_string());
        let record2 = DataRecord::new(2, 20.0, "B".to_string());
        
        processor.add_record(record1);
        processor.add_record(record2);
        
        assert_eq!(processor.get_stats().total_records, 2);
        assert_eq!(processor.get_stats().valid_records, 2);
        assert_eq!(processor.get_stats().average_value, 15.0);
    }

    #[test]
    fn test_file_loading() -> Result<(), Box<dyn Error>> {
        let mut file = NamedTempFile::new()?;
        writeln!(file, "1,10.5,category1")?;
        writeln!(file, "2,20.3,category2")?;
        writeln!(file, "# This is a comment")?;
        writeln!(file, "")?;
        writeln!(file, "3,15.7,category1")?;
        
        let mut processor = DataProcessor::new();
        processor.load_from_file(file.path())?;
        
        assert_eq!(processor.get_stats().total_records, 3);
        assert_eq!(processor.get_stats().valid_records, 3);
        assert_eq!(processor.filter_by_category("category1").len(), 2);
        
        Ok(())
    }
}