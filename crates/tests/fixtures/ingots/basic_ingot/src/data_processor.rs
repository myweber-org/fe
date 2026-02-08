
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

        for (line_number, line) in reader.lines().enumerate() {
            let line = line?;
            
            if line.trim().is_empty() {
                continue;
            }

            if self.has_header && line_number == 0 {
                continue;
            }

            let fields: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            if !self.validate_record(&fields) {
                return Err(format!("Invalid record at line {}", line_number + 1).into());
            }

            records.push(fields);
        }

        Ok(records)
    }

    fn validate_record(&self, fields: &[String]) -> bool {
        !fields.is_empty() && fields.iter().all(|f| !f.is_empty())
    }

    pub fn calculate_statistics(&self, data: &[Vec<String>], column_index: usize) -> Option<(f64, f64, f64)> {
        if data.is_empty() {
            return None;
        }

        let mut values = Vec::new();
        for record in data {
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
    fn test_data_processor() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,salary").unwrap();
        writeln!(temp_file, "Alice,30,50000").unwrap();
        writeln!(temp_file, "Bob,25,45000").unwrap();
        writeln!(temp_file, "Charlie,35,60000").unwrap();

        let processor = DataProcessor::new(',', true);
        let result = processor.process_file(temp_file.path());
        
        assert!(result.is_ok());
        let data = result.unwrap();
        assert_eq!(data.len(), 3);
        
        let stats = processor.calculate_statistics(&data, 1);
        assert!(stats.is_some());
        
        let (mean, _, _) = stats.unwrap();
        assert!((mean - 30.0).abs() < 0.001);
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
        if threshold <= 0.0 {
            return Err(ValidationError {
                message: "Threshold must be positive".to_string(),
            });
        }
        Ok(DataProcessor { threshold })
    }

    pub fn process_values(&self, values: &[f64]) -> Result<Vec<f64>, ValidationError> {
        if values.is_empty() {
            return Err(ValidationError {
                message: "Input values cannot be empty".to_string(),
            });
        }

        let filtered: Vec<f64> = values
            .iter()
            .filter(|&&v| v >= self.threshold)
            .cloned()
            .collect();

        if filtered.is_empty() {
            return Err(ValidationError {
                message: format!("No values above threshold {}", self.threshold),
            });
        }

        let normalized = self.normalize_data(&filtered);
        Ok(normalized)
    }

    fn normalize_data(&self, data: &[f64]) -> Vec<f64> {
        let max_value = data
            .iter()
            .fold(f64::NEG_INFINITY, |a, &b| a.max(b));

        data.iter()
            .map(|&v| v / max_value)
            .collect()
    }

    pub fn calculate_statistics(&self, data: &[f64]) -> (f64, f64, f64) {
        let count = data.len() as f64;
        let sum: f64 = data.iter().sum();
        let mean = sum / count;

        let variance: f64 = data
            .iter()
            .map(|&value| {
                let diff = mean - value;
                diff * diff
            })
            .sum::<f64>()
            / count;

        let std_dev = variance.sqrt();

        (mean, variance, std_dev)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_processor_creation() {
        let processor = DataProcessor::new(5.0);
        assert!(processor.is_ok());
    }

    #[test]
    fn test_invalid_processor_creation() {
        let processor = DataProcessor::new(0.0);
        assert!(processor.is_err());
    }

    #[test]
    fn test_process_values() {
        let processor = DataProcessor::new(2.0).unwrap();
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let result = processor.process_values(&values);
        assert!(result.is_ok());

        let processed = result.unwrap();
        assert_eq!(processed.len(), 4);
        assert!(!processed.contains(&1.0));
    }

    #[test]
    fn test_calculate_statistics() {
        let processor = DataProcessor::new(1.0).unwrap();
        let data = vec![2.0, 4.0, 6.0, 8.0];
        let (mean, variance, std_dev) = processor.calculate_statistics(&data);

        assert_eq!(mean, 5.0);
        assert_eq!(variance, 5.0);
        assert_eq!(std_dev, 5.0_f64.sqrt());
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

    pub fn get_statistics(&self) -> Statistics {
        let count = self.records.len();
        let avg = self.calculate_average().unwrap_or(0.0);
        let max = self
            .records
            .iter()
            .map(|r| r.value)
            .fold(f64::NEG_INFINITY, f64::max);
        let min = self
            .records
            .iter()
            .map(|r| r.value)
            .fold(f64::INFINITY, f64::min);

        Statistics {
            count,
            average: avg,
            max_value: max,
            min_value: min,
        }
    }
}

pub struct Statistics {
    pub count: usize,
    pub average: f64,
    pub max_value: f64,
    pub min_value: f64,
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
        writeln!(temp_file, "id,name,value,category").unwrap();
        writeln!(temp_file, "1,ItemA,10.5,Category1").unwrap();
        writeln!(temp_file, "2,ItemB,20.3,Category2").unwrap();
        writeln!(temp_file, "3,ItemC,15.7,Category1").unwrap();
        
        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        
        let filtered = processor.filter_by_category("Category1");
        assert_eq!(filtered.len(), 2);
        
        let avg = processor.calculate_average();
        assert!(avg.is_some());
        assert!((avg.unwrap() - 15.5).abs() < 0.1);
        
        let stats = processor.get_statistics();
        assert_eq!(stats.count, 3);
        assert_eq!(stats.max_value, 20.3);
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
            return Err(format!("Invalid value {} for record {}", value, id));
        }
        if category.is_empty() {
            return Err(format!("Empty category for record {}", id));
        }
        Ok(Self { id, value, category })
    }
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        Self { records: Vec::new() }
    }

    pub fn load_from_csv(&mut self, file_path: &Path) -> Result<usize, Box<dyn Error>> {
        let file = File::open(file_path)?;
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
                Ok(id) => id,
                Err(_) => continue,
            };

            let value = match parts[1].parse::<f64>() {
                Ok(value) => value,
                Err(_) => continue,
            };

            let category = parts[2].trim().to_string();

            match DataRecord::new(id, value, category) {
                Ok(record) => {
                    self.records.push(record);
                    loaded_count += 1;
                }
                Err(e) => eprintln!("Skipping record {}: {}", id, e),
            }
        }

        Ok(loaded_count)
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

    pub fn total_records(&self) -> usize {
        self.records.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_valid_record_creation() {
        let record = DataRecord::new(1, 42.5, "test".to_string());
        assert!(record.is_ok());
        let record = record.unwrap();
        assert_eq!(record.id, 1);
        assert_eq!(record.value, 42.5);
        assert_eq!(record.category, "test");
    }

    #[test]
    fn test_invalid_record_creation() {
        let record = DataRecord::new(2, -5.0, "test".to_string());
        assert!(record.is_err());
    }

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        assert_eq!(processor.total_records(), 0);

        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category").unwrap();
        writeln!(temp_file, "1,10.5,alpha").unwrap();
        writeln!(temp_file, "2,20.0,beta").unwrap();
        writeln!(temp_file, "3,15.75,alpha").unwrap();

        let result = processor.load_from_csv(temp_file.path());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 3);
        assert_eq!(processor.total_records(), 3);

        let avg = processor.calculate_average();
        assert!(avg.is_some());
        assert!((avg.unwrap() - 15.416666666666666).abs() < 0.0001);

        let alpha_records = processor.filter_by_category("alpha");
        assert_eq!(alpha_records.len(), 2);
    }
}
use csv::{Reader, Writer};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
pub struct Record {
    id: u32,
    name: String,
    value: f64,
    active: bool,
}

impl Record {
    pub fn new(id: u32, name: String, value: f64, active: bool) -> Self {
        Record {
            id,
            name,
            value,
            active,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("Name cannot be empty".to_string());
        }
        if self.value < 0.0 {
            return Err("Value must be non-negative".to_string());
        }
        Ok(())
    }
}

pub struct DataProcessor;

impl DataProcessor {
    pub fn load_from_csv<P: AsRef<Path>>(path: P) -> Result<Vec<Record>, Box<dyn Error>> {
        let mut reader = Reader::from_path(path)?;
        let mut records = Vec::new();

        for result in reader.deserialize() {
            let record: Record = result?;
            record.validate()?;
            records.push(record);
        }

        Ok(records)
    }

    pub fn save_to_csv<P: AsRef<Path>>(records: &[Record], path: P) -> Result<(), Box<dyn Error>> {
        let mut writer = Writer::from_path(path)?;

        for record in records {
            writer.serialize(record)?;
        }

        writer.flush()?;
        Ok(())
    }

    pub fn filter_active(records: &[Record]) -> Vec<&Record> {
        records.iter().filter(|r| r.active).collect()
    }

    pub fn calculate_total(records: &[Record]) -> f64 {
        records.iter().map(|r| r.value).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_record_validation() {
        let valid_record = Record::new(1, "Test".to_string(), 100.0, true);
        assert!(valid_record.validate().is_ok());

        let invalid_record = Record::new(2, "".to_string(), -50.0, false);
        assert!(invalid_record.validate().is_err());
    }

    #[test]
    fn test_csv_operations() -> Result<(), Box<dyn Error>> {
        let records = vec![
            Record::new(1, "Alice".to_string(), 150.0, true),
            Record::new(2, "Bob".to_string(), 200.0, false),
            Record::new(3, "Charlie".to_string(), 300.0, true),
        ];

        let temp_file = NamedTempFile::new()?;
        let path = temp_file.path();

        DataProcessor::save_to_csv(&records, path)?;
        let loaded_records = DataProcessor::load_from_csv(path)?;

        assert_eq!(loaded_records.len(), 3);
        assert_eq!(DataProcessor::calculate_total(&loaded_records), 650.0);
        assert_eq!(DataProcessor::filter_active(&loaded_records).len(), 2);

        Ok(())
    }
}