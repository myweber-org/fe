
use std::collections::HashMap;
use std::error::Error;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub timestamp: i64,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

impl DataRecord {
    pub fn new(id: u32, timestamp: i64, values: Vec<f64>) -> Self {
        Self {
            id,
            timestamp,
            values,
            metadata: HashMap::new(),
        }
    }

    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    pub fn validate(&self) -> Result<(), Box<dyn Error>> {
        if self.id == 0 {
            return Err("Invalid record ID".into());
        }
        if self.timestamp < 0 {
            return Err("Invalid timestamp".into());
        }
        if self.values.is_empty() {
            return Err("Empty values array".into());
        }
        Ok(())
    }
}

pub fn process_records(records: Vec<DataRecord>) -> Vec<DataRecord> {
    records
        .into_iter()
        .filter(|record| record.validate().is_ok())
        .map(|mut record| {
            let transformed_values: Vec<f64> = record
                .values
                .iter()
                .map(|&value| value * 2.0)
                .collect();
            record.values = transformed_values;
            record.add_metadata("processed".to_string(), "true".to_string());
            record
        })
        .collect()
}

pub fn calculate_statistics(records: &[DataRecord]) -> HashMap<String, f64> {
    let mut stats = HashMap::new();
    
    if records.is_empty() {
        return stats;
    }

    let total_values: usize = records.iter().map(|r| r.values.len()).sum();
    let sum_all: f64 = records.iter().flat_map(|r| &r.values).sum();
    let count_all = total_values as f64;

    stats.insert("mean".to_string(), sum_all / count_all);
    stats.insert("total_records".to_string(), records.len() as f64);
    stats.insert("total_values".to_string(), total_values as f64);

    stats
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_validation() {
        let valid_record = DataRecord::new(1, 1234567890, vec![1.0, 2.0, 3.0]);
        assert!(valid_record.validate().is_ok());

        let invalid_record = DataRecord::new(0, 1234567890, vec![1.0]);
        assert!(invalid_record.validate().is_err());
    }

    #[test]
    fn test_process_records() {
        let records = vec![
            DataRecord::new(1, 1000, vec![1.0, 2.0]),
            DataRecord::new(2, 2000, vec![3.0, 4.0]),
        ];
        
        let processed = process_records(records);
        assert_eq!(processed.len(), 2);
        assert_eq!(processed[0].values, vec![2.0, 4.0]);
        assert_eq!(processed[1].metadata.get("processed"), Some(&"true".to_string()));
    }

    #[test]
    fn test_calculate_statistics() {
        let records = vec![
            DataRecord::new(1, 1000, vec![1.0, 2.0]),
            DataRecord::new(2, 2000, vec![3.0, 4.0]),
        ];
        
        let stats = calculate_statistics(&records);
        assert_eq!(stats.get("mean"), Some(&2.5));
        assert_eq!(stats.get("total_records"), Some(&2.0));
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
    let mut reader = Reader::from_reader(file);
    
    let mut records = Vec::new();
    for result in reader.deserialize() {
        let record: Record = result?;
        validate_record(&record)?;
        records.push(record);
    }
    
    Ok(records)
}

fn validate_record(record: &Record) -> Result<(), String> {
    if record.id == 0 {
        return Err("ID cannot be zero".to_string());
    }
    
    if record.name.is_empty() {
        return Err("Name cannot be empty".to_string());
    }
    
    if record.value < 0.0 {
        return Err("Value cannot be negative".to_string());
    }
    
    let valid_categories = ["A", "B", "C", "D"];
    if !valid_categories.contains(&record.category.as_str()) {
        return Err(format!("Invalid category: {}", record.category));
    }
    
    Ok(())
}

pub fn calculate_statistics(records: &[Record]) -> (f64, f64, f64) {
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
    use tempfile::NamedTempFile;
    use std::io::Write;
    
    #[test]
    fn test_process_valid_data() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,category").unwrap();
        writeln!(temp_file, "1,Test1,100.5,A").unwrap();
        writeln!(temp_file, "2,Test2,200.0,B").unwrap();
        
        let result = process_data_file(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 2);
    }
    
    #[test]
    fn test_validate_record() {
        let valid_record = Record {
            id: 1,
            name: "Test".to_string(),
            value: 50.0,
            category: "C".to_string(),
        };
        assert!(validate_record(&valid_record).is_ok());
        
        let invalid_record = Record {
            id: 0,
            name: "".to_string(),
            value: -10.0,
            category: "X".to_string(),
        };
        assert!(validate_record(&invalid_record).is_err());
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
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
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

    pub fn calculate_tax(&self, rate: f64) -> f64 {
        self.value * rate
    }
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        Self { records: Vec::new() }
    }

    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<usize, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut count = 0;

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
            let category = parts[2].to_string();

            match DataRecord::new(id, value, category) {
                Ok(record) => {
                    self.records.push(record);
                    count += 1;
                }
                Err(e) => eprintln!("Warning: Skipping line {}: {}", line_num + 1, e),
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

    pub fn process_all_taxes(&self, rate: f64) -> Vec<(u32, f64)> {
        self.records
            .iter()
            .map(|r| (r.id, r.calculate_tax(rate)))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_record_creation() {
        let record = DataRecord::new(1, 100.0, "A".to_string()).unwrap();
        assert_eq!(record.id, 1);
        assert_eq!(record.value, 100.0);
        assert_eq!(record.category, "A");
    }

    #[test]
    fn test_invalid_record() {
        assert!(DataRecord::new(1, -10.0, "A".to_string()).is_err());
        assert!(DataRecord::new(1, 10.0, "".to_string()).is_err());
    }

    #[test]
    fn test_tax_calculation() {
        let record = DataRecord::new(1, 100.0, "A".to_string()).unwrap();
        assert_eq!(record.calculate_tax(0.1), 10.0);
    }

    #[test]
    fn test_data_processor() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "1,100.0,CategoryA").unwrap();
        writeln!(file, "2,200.0,CategoryB").unwrap();
        writeln!(file, "3,300.0,CategoryA").unwrap();

        let mut processor = DataProcessor::new();
        let count = processor.load_from_file(file.path()).unwrap();
        assert_eq!(count, 3);
        assert_eq!(processor.total_value(), 600.0);
        assert_eq!(processor.average_value(), Some(200.0));

        let category_a = processor.filter_by_category("CategoryA");
        assert_eq!(category_a.len(), 2);

        let taxes = processor.process_all_taxes(0.1);
        assert_eq!(taxes.len(), 3);
        assert_eq!(taxes[0], (1, 10.0));
    }
}
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct DataRecord {
    id: u64,
    name: String,
    value: f64,
    tags: Vec<String>,
    metadata: HashMap<String, String>,
}

impl DataRecord {
    pub fn new(id: u64, name: String, value: f64) -> Self {
        Self {
            id,
            name,
            value,
            tags: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("Name cannot be empty".to_string());
        }
        if self.value < 0.0 {
            return Err("Value must be non-negative".to_string());
        }
        if self.id == 0 {
            return Err("ID must be non-zero".to_string());
        }
        Ok(())
    }

    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
    }

    pub fn set_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    pub fn transform_value<F>(&mut self, transformer: F)
    where
        F: Fn(f64) -> f64,
    {
        self.value = transformer(self.value);
    }
}

pub fn process_records(records: &mut [DataRecord]) -> Vec<Result<(), String>> {
    records
        .iter_mut()
        .map(|record| {
            record.validate()?;
            record.transform_value(|v| v * 1.1);
            record.add_tag("processed".to_string());
            record.set_metadata("processed_at".to_string(), chrono::Utc::now().to_rfc3339());
            Ok(())
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_validation() {
        let valid_record = DataRecord::new(1, "test".to_string(), 100.0);
        assert!(valid_record.validate().is_ok());

        let invalid_record = DataRecord::new(0, "".to_string(), -10.0);
        assert!(invalid_record.validate().is_err());
    }

    #[test]
    fn test_value_transformation() {
        let mut record = DataRecord::new(1, "test".to_string(), 100.0);
        record.transform_value(|v| v * 2.0);
        assert_eq!(record.value, 200.0);
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
    category: String,
}

pub fn process_data_file(path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(path)?;
    let mut rdr = Reader::from_reader(file);
    let mut records = Vec::new();

    for result in rdr.deserialize() {
        let record: Record = result?;
        if record.value >= 0.0 && record.category.len() <= 20 {
            records.push(record);
        }
    }

    Ok(records)
}

pub fn calculate_statistics(records: &[Record]) -> (f64, f64, usize) {
    let sum: f64 = records.iter().map(|r| r.value).sum();
    let count = records.len();
    let mean = if count > 0 { sum / count as f64 } else { 0.0 };
    let max = records.iter().map(|r| r.value).fold(0.0, f64::max);
    
    (mean, max, count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_process_valid_data() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,category").unwrap();
        writeln!(temp_file, "1,Test1,10.5,CategoryA").unwrap();
        writeln!(temp_file, "2,Test2,15.3,CategoryB").unwrap();
        
        let records = process_data_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].name, "Test1");
    }

    #[test]
    fn test_statistics_calculation() {
        let records = vec![
            Record { id: 1, name: "A".to_string(), value: 10.0, category: "X".to_string() },
            Record { id: 2, name: "B".to_string(), value: 20.0, category: "Y".to_string() },
            Record { id: 3, name: "C".to_string(), value: 30.0, category: "Z".to_string() },
        ];
        
        let (mean, max, count) = calculate_statistics(&records);
        assert_eq!(mean, 20.0);
        assert_eq!(max, 30.0);
        assert_eq!(count, 3);
    }
}
use csv::{Reader, Writer};
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
}

pub fn process_csv(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let mut reader = Reader::from_reader(input_file);
    
    let output_file = File::create(output_path)?;
    let mut writer = Writer::from_writer(output_file);
    
    let mut valid_count = 0;
    let mut invalid_count = 0;
    
    for result in reader.deserialize() {
        let record: Record = result?;
        
        if record.is_valid() {
            writer.serialize(&record)?;
            valid_count += 1;
        } else {
            invalid_count += 1;
        }
    }
    
    writer.flush()?;
    
    println!("Processing complete:");
    println!("  Valid records: {}", valid_count);
    println!("  Invalid records: {}", invalid_count);
    
    Ok(())
}

pub fn calculate_average_value(records: &[Record]) -> f64 {
    if records.is_empty() {
        return 0.0;
    }
    
    let sum: f64 = records.iter().map(|r| r.value).sum();
    sum / records.len() as f64
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_valid_record() {
        let record = Record {
            id: 1,
            name: "Test".to_string(),
            value: 10.5,
            active: true,
        };
        assert!(record.is_valid());
    }
    
    #[test]
    fn test_invalid_record_empty_name() {
        let record = Record {
            id: 2,
            name: "".to_string(),
            value: 5.0,
            active: false,
        };
        assert!(!record.is_valid());
    }
    
    #[test]
    fn test_calculate_average() {
        let records = vec![
            Record { id: 1, name: "A".to_string(), value: 10.0, active: true },
            Record { id: 2, name: "B".to_string(), value: 20.0, active: false },
            Record { id: 3, name: "C".to_string(), value: 30.0, active: true },
        ];
        
        assert_eq!(calculate_average_value(&records), 20.0);
    }
}