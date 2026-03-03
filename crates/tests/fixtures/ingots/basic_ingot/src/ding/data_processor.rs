
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

    pub fn calculate_average(&self, column_name: &str) -> Option<f64> {
        let mut sum = 0.0;
        let mut count = 0;

        for record in &self.records {
            if let Some(value_str) = record.get(column_name) {
                if let Ok(value) = value_str.parse::<f64>() {
                    sum += value;
                    count += 1;
                }
            }
        }

        if count > 0 {
            Some(sum / count as f64)
        } else {
            None
        }
    }

    pub fn count_unique_values(&self, column_name: &str) -> HashMap<String, usize> {
        let mut frequency_map = HashMap::new();
        
        for record in &self.records {
            if let Some(value) = record.get(column_name) {
                *frequency_map.entry(value.clone()).or_insert(0) += 1;
            }
        }
        
        frequency_map
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

    pub fn get_record_count(&self) -> usize {
        self.records.len()
    }

    pub fn get_column_names(&self) -> Vec<String> {
        if let Some(first_record) = self.records.first() {
            first_record.keys().cloned().collect()
        } else {
            Vec::new()
        }
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
        
        assert_eq!(processor.get_record_count(), 3);
        
        let avg_age = processor.calculate_average("age");
        assert_eq!(avg_age, Some(26.666666666666668));
        
        let age_frequency = processor.count_unique_values("age");
        assert_eq!(age_frequency.get("25"), Some(&2));
        assert_eq!(age_frequency.get("30"), Some(&1));
        
        let filtered = processor.filter_records(|record| {
            record.get("age").map_or(false, |age| age == "25")
        });
        assert_eq!(filtered.len(), 2);
        
        let columns = processor.get_column_names();
        assert_eq!(columns, vec!["name", "age", "score"]);
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

pub fn calculate_statistics(records: &[Record]) -> (f64, f64, usize) {
    let sum: f64 = records.iter().map(|r| r.value).sum();
    let count = records.len();
    let average = if count > 0 { sum / count as f64 } else { 0.0 };
    
    let max_value = records.iter()
        .map(|r| r.value)
        .fold(f64::NEG_INFINITY, f64::max);
    
    (average, max_value, count)
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
    fn test_process_csv() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,category").unwrap();
        writeln!(temp_file, "1,ItemA,10.5,Category1").unwrap();
        writeln!(temp_file, "2,ItemB,-5.0,Category2").unwrap();
        writeln!(temp_file, "3,ItemC,20.0,Category1").unwrap();
        
        let records = process_csv_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].name, "ItemA");
        assert_eq!(records[1].value, 20.0);
    }

    #[test]
    fn test_calculate_statistics() {
        let records = vec![
            Record { id: 1, name: "Test1".to_string(), value: 10.0, category: "A".to_string() },
            Record { id: 2, name: "Test2".to_string(), value: 20.0, category: "B".to_string() },
            Record { id: 3, name: "Test3".to_string(), value: 30.0, category: "A".to_string() },
        ];
        
        let (avg, max, count) = calculate_statistics(&records);
        assert_eq!(avg, 20.0);
        assert_eq!(max, 30.0);
        assert_eq!(count, 3);
    }

    #[test]
    fn test_filter_by_category() {
        let records = vec![
            Record { id: 1, name: "X".to_string(), value: 10.0, category: "Cat1".to_string() },
            Record { id: 2, name: "Y".to_string(), value: 20.0, category: "Cat2".to_string() },
            Record { id: 3, name: "Z".to_string(), value: 30.0, category: "Cat1".to_string() },
        ];
        
        let filtered = filter_by_category(records, "Cat1");
        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().all(|r| r.category == "Cat1"));
    }
}use std::error::Error;
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
                message: format!("Threshold {} must be between 0.0 and 1.0", threshold),
            });
        }
        Ok(Self { threshold })
    }

    pub fn process_values(&self, values: &[f64]) -> Vec<f64> {
        values
            .iter()
            .filter(|&&v| v >= self.threshold)
            .map(|&v| v * 2.0)
            .collect()
    }

    pub fn calculate_statistics(&self, values: &[f64]) -> (f64, f64, f64) {
        if values.is_empty() {
            return (0.0, 0.0, 0.0);
        }

        let sum: f64 = values.iter().sum();
        let mean = sum / values.len() as f64;
        
        let variance: f64 = values
            .iter()
            .map(|&value| {
                let diff = mean - value;
                diff * diff
            })
            .sum::<f64>() / values.len() as f64;
        
        let std_dev = variance.sqrt();
        
        (mean, variance, std_dev)
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
    fn test_invalid_processor_creation() {
        let processor = DataProcessor::new(1.5);
        assert!(processor.is_err());
    }

    #[test]
    fn test_process_values() {
        let processor = DataProcessor::new(0.3).unwrap();
        let values = vec![0.1, 0.4, 0.2, 0.5, 0.6];
        let result = processor.process_values(&values);
        assert_eq!(result, vec![0.8, 1.0, 1.2]);
    }

    #[test]
    fn test_calculate_statistics() {
        let processor = DataProcessor::new(0.0).unwrap();
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let (mean, variance, std_dev) = processor.calculate_statistics(&values);
        
        assert!((mean - 3.0).abs() < 0.0001);
        assert!((variance - 2.0).abs() < 0.0001);
        assert!((std_dev - 1.41421356237).abs() < 0.0001);
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
    InvalidCategory,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::InvalidId => write!(f, "ID must be greater than 0"),
            ValidationError::EmptyName => write!(f, "Name cannot be empty"),
            ValidationError::NegativeValue => write!(f, "Value must be non-negative"),
            ValidationError::InvalidCategory => write!(f, "Category must be one of: A, B, C"),
        }
    }
}

impl Error for ValidationError {}

pub fn validate_record(record: &DataRecord) -> Result<(), ValidationError> {
    if record.id == 0 {
        return Err(ValidationError::InvalidId);
    }
    
    if record.name.trim().is_empty() {
        return Err(ValidationError::EmptyName);
    }
    
    if record.value < 0.0 {
        return Err(ValidationError::NegativeValue);
    }
    
    let valid_categories = ["A", "B", "C"];
    if !valid_categories.contains(&record.category.as_str()) {
        return Err(ValidationError::InvalidCategory);
    }
    
    Ok(())
}

pub fn transform_records(records: Vec<DataRecord>) -> HashMap<String, Vec<DataRecord>> {
    let mut grouped = HashMap::new();
    
    for record in records {
        grouped
            .entry(record.category.clone())
            .or_insert_with(Vec::new)
            .push(record);
    }
    
    grouped
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
    
    #[test]
    fn test_validate_record_valid() {
        let record = DataRecord {
            id: 1,
            name: "Test".to_string(),
            value: 10.5,
            category: "A".to_string(),
        };
        
        assert!(validate_record(&record).is_ok());
    }
    
    #[test]
    fn test_validate_record_invalid_id() {
        let record = DataRecord {
            id: 0,
            name: "Test".to_string(),
            value: 10.5,
            category: "A".to_string(),
        };
        
        assert!(matches!(validate_record(&record), Err(ValidationError::InvalidId)));
    }
    
    #[test]
    fn test_transform_records() {
        let records = vec![
            DataRecord {
                id: 1,
                name: "Item1".to_string(),
                value: 10.0,
                category: "A".to_string(),
            },
            DataRecord {
                id: 2,
                name: "Item2".to_string(),
                value: 20.0,
                category: "B".to_string(),
            },
            DataRecord {
                id: 3,
                name: "Item3".to_string(),
                value: 30.0,
                category: "A".to_string(),
            },
        ];
        
        let grouped = transform_records(records);
        
        assert_eq!(grouped.get("A").unwrap().len(), 2);
        assert_eq!(grouped.get("B").unwrap().len(), 1);
        assert!(grouped.get("C").is_none());
    }
    
    #[test]
    fn test_calculate_statistics() {
        let records = vec![
            DataRecord {
                id: 1,
                name: "Item1".to_string(),
                value: 10.0,
                category: "A".to_string(),
            },
            DataRecord {
                id: 2,
                name: "Item2".to_string(),
                value: 20.0,
                category: "A".to_string(),
            },
            DataRecord {
                id: 3,
                name: "Item3".to_string(),
                value: 30.0,
                category: "A".to_string(),
            },
        ];
        
        let (mean, variance, std_dev) = calculate_statistics(&records);
        
        assert_eq!(mean, 20.0);
        assert_eq!(variance, 66.66666666666667);
        assert_eq!(std_dev, 8.16496580927726);
    }
}