use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;

#[derive(Debug)]
pub struct DataRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub category: String,
}

impl DataRecord {
    pub fn new(id: u32, name: String, value: f64, category: String) -> Self {
        DataRecord {
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
    category_totals: HashMap<String, f64>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
            category_totals: HashMap::new(),
        }
    }

    pub fn load_from_csv(&mut self, filepath: &str) -> Result<usize, Box<dyn Error>> {
        let file = File::open(filepath)?;
        let reader = BufReader::new(file);
        let mut count = 0;

        for (index, line) in reader.lines().enumerate() {
            if index == 0 {
                continue;
            }

            let line = line?;
            let parts: Vec<&str> = line.split(',').collect();
            
            if parts.len() == 4 {
                let id = parts[0].parse::<u32>()?;
                let name = parts[1].to_string();
                let value = parts[2].parse::<f64>()?;
                let category = parts[3].to_string();

                let record = DataRecord::new(id, name, value, category);
                
                if record.is_valid() {
                    self.records.push(record);
                    count += 1;
                }
            }
        }

        self.calculate_totals();
        Ok(count)
    }

    fn calculate_totals(&mut self) {
        self.category_totals.clear();
        
        for record in &self.records {
            *self.category_totals.entry(record.category.clone())
                .or_insert(0.0) += record.value;
        }
    }

    pub fn get_category_total(&self, category: &str) -> Option<f64> {
        self.category_totals.get(category).copied()
    }

    pub fn get_average_value(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }
        
        let total: f64 = self.records.iter().map(|r| r.value).sum();
        total / self.records.len() as f64
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records.iter()
            .filter(|record| record.category == category)
            .collect()
    }

    pub fn get_statistics(&self) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        
        if !self.records.is_empty() {
            let values: Vec<f64> = self.records.iter().map(|r| r.value).collect();
            let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
            let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
            
            stats.insert("min".to_string(), min);
            stats.insert("max".to_string(), max);
            stats.insert("average".to_string(), self.get_average_value());
            stats.insert("count".to_string(), self.records.len() as f64);
        }
        
        stats
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

        let invalid_record = DataRecord::new(2, "".to_string(), -5.0, "".to_string());
        assert!(!invalid_record.is_valid());
    }

    #[test]
    fn test_data_processing() {
        let mut processor = DataProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,category").unwrap();
        writeln!(temp_file, "1,Item1,10.5,CategoryA").unwrap();
        writeln!(temp_file, "2,Item2,20.0,CategoryB").unwrap();
        writeln!(temp_file, "3,Item3,15.75,CategoryA").unwrap();
        
        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 3);
        
        assert_eq!(processor.get_category_total("CategoryA"), Some(26.25));
        assert_eq!(processor.get_average_value(), 15.416666666666666);
        
        let category_a_items = processor.filter_by_category("CategoryA");
        assert_eq!(category_a_items.len(), 2);
    }
}
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProcessingError {
    #[error("Invalid input data")]
    InvalidData,
    #[error("Transformation failed")]
    TransformationFailed,
    #[error("Validation error: {0}")]
    ValidationError(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRecord {
    pub id: u64,
    pub timestamp: i64,
    pub values: HashMap<String, f64>,
    pub tags: Vec<String>,
}

impl DataRecord {
    pub fn new(id: u64, timestamp: i64) -> Self {
        Self {
            id,
            timestamp,
            values: HashMap::new(),
            tags: Vec::new(),
        }
    }

    pub fn add_value(&mut self, key: String, value: f64) {
        self.values.insert(key, value);
    }

    pub fn add_tag(&mut self, tag: String) {
        self.tags.push(tag);
    }

    pub fn validate(&self) -> Result<(), ProcessingError> {
        if self.id == 0 {
            return Err(ProcessingError::ValidationError(
                "ID cannot be zero".to_string(),
            ));
        }

        if self.timestamp < 0 {
            return Err(ProcessingError::ValidationError(
                "Timestamp cannot be negative".to_string(),
            ));
        }

        if self.values.is_empty() {
            return Err(ProcessingError::ValidationError(
                "Record must contain at least one value".to_string(),
            ));
        }

        for (key, value) in &self.values {
            if key.trim().is_empty() {
                return Err(ProcessingError::ValidationError(
                    "Value key cannot be empty".to_string(),
                ));
            }
            if !value.is_finite() {
                return Err(ProcessingError::ValidationError(
                    format!("Value for '{}' must be finite", key),
                ));
            }
        }

        Ok(())
    }

    pub fn transform(&mut self, multiplier: f64) -> Result<(), ProcessingError> {
        if !multiplier.is_finite() || multiplier == 0.0 {
            return Err(ProcessingError::TransformationFailed);
        }

        for value in self.values.values_mut() {
            *value *= multiplier;
        }

        Ok(())
    }
}

pub fn process_records(
    records: Vec<DataRecord>,
    multiplier: f64,
) -> Result<Vec<DataRecord>, ProcessingError> {
    let mut processed = Vec::with_capacity(records.len());

    for mut record in records {
        record.validate()?;
        record.transform(multiplier)?;
        processed.push(record);
    }

    Ok(processed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_record() {
        let mut record = DataRecord::new(1, 1234567890);
        record.add_value("temperature".to_string(), 25.5);
        record.add_tag("sensor".to_string());

        assert!(record.validate().is_ok());
    }

    #[test]
    fn test_invalid_record_empty_values() {
        let record = DataRecord::new(1, 1234567890);
        assert!(record.validate().is_err());
    }

    #[test]
    fn test_transform_values() {
        let mut record = DataRecord::new(1, 1234567890);
        record.add_value("pressure".to_string(), 100.0);
        record.add_value("humidity".to_string(), 50.0);

        assert!(record.transform(2.0).is_ok());
        assert_eq!(record.values.get("pressure"), Some(&200.0));
        assert_eq!(record.values.get("humidity"), Some(&100.0));
    }

    #[test]
    fn test_process_multiple_records() {
        let mut record1 = DataRecord::new(1, 1000);
        record1.add_value("temp".to_string(), 10.0);

        let mut record2 = DataRecord::new(2, 2000);
        record2.add_value("temp".to_string(), 20.0);

        let records = vec![record1, record2];
        let result = process_records(records, 3.0);

        assert!(result.is_ok());
        let processed = result.unwrap();
        assert_eq!(processed.len(), 2);
        assert_eq!(processed[0].values.get("temp"), Some(&30.0));
        assert_eq!(processed[1].values.get("temp"), Some(&60.0));
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

    pub fn process_csv<P: AsRef<Path>>(&self, file_path: P) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();
        let mut lines = reader.lines();

        if self.has_header {
            let _header = lines.next().transpose()?;
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

    pub fn validate_numeric_fields(&self, records: &[Vec<String>], field_index: usize) -> Result<Vec<f64>, Box<dyn Error>> {
        let mut numeric_values = Vec::new();
        
        for (row_num, record) in records.iter().enumerate() {
            if field_index >= record.len() {
                return Err(format!("Row {}: Field index {} out of bounds", row_num + 1, field_index).into());
            }
            
            match record[field_index].parse::<f64>() {
                Ok(value) => numeric_values.push(value),
                Err(_) => return Err(format!("Row {}: Invalid numeric value '{}'", row_num + 1, record[field_index]).into()),
            }
        }
        
        Ok(numeric_values)
    }

    pub fn calculate_statistics(&self, values: &[f64]) -> (f64, f64, f64) {
        if values.is_empty() {
            return (0.0, 0.0, 0.0);
        }

        let sum: f64 = values.iter().sum();
        let mean = sum / values.len() as f64;
        
        let variance: f64 = values.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / values.len() as f64;
        
        let std_dev = variance.sqrt();
        
        (mean, variance, std_dev)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_processing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,salary").unwrap();
        writeln!(temp_file, "Alice,30,50000.0").unwrap();
        writeln!(temp_file, "Bob,25,45000.5").unwrap();
        
        let processor = DataProcessor::new(',', true);
        let records = processor.process_csv(temp_file.path()).unwrap();
        
        assert_eq!(records.len(), 2);
        assert_eq!(records[0], vec!["Alice", "30", "50000.0"]);
    }

    #[test]
    fn test_numeric_validation() {
        let records = vec![
            vec!["10.5".to_string(), "text".to_string()],
            vec!["20.0".to_string(), "more".to_string()],
        ];
        
        let processor = DataProcessor::new(',', false);
        let numeric_values = processor.validate_numeric_fields(&records, 0).unwrap();
        
        assert_eq!(numeric_values, vec![10.5, 20.0]);
    }

    #[test]
    fn test_statistics_calculation() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let processor = DataProcessor::new(',', false);
        let (mean, variance, std_dev) = processor.calculate_statistics(&values);
        
        assert_eq!(mean, 3.0);
        assert_eq!(variance, 2.0);
        assert_eq!(std_dev, 2.0_f64.sqrt());
    }
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

pub fn process_csv_data(input_path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let mut reader = Reader::from_path(input_path)?;
    let mut records = Vec::new();

    for result in reader.deserialize() {
        let record: Record = result?;
        
        if record.value < 0.0 {
            return Err(format!("Invalid value {} for record {}", record.value, record.id).into());
        }
        
        if !record.category.is_empty() {
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
        writeln!(file, "1,Test1,10.5,CategoryA").unwrap();
        writeln!(file, "2,Test2,20.0,CategoryB").unwrap();
        
        let result = process_csv_data(file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 2);
    }

    #[test]
    fn test_statistics_calculation() {
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
}