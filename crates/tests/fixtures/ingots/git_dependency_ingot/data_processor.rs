
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

        for (index, line) in reader.lines().enumerate() {
            let line = line?;
            
            if self.has_header && index == 0 {
                continue;
            }

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

    pub fn extract_column(&self, data: &[Vec<String>], column_index: usize) -> Vec<String> {
        data.iter()
            .filter_map(|record| record.get(column_index).cloned())
            .collect()
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
        writeln!(temp_file, "Alice,30,New York").unwrap();
        writeln!(temp_file, "Bob,25,London").unwrap();

        let processor = DataProcessor::new(',', true);
        let result = processor.process_file(temp_file.path()).unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["Alice", "30", "New York"]);
    }

    #[test]
    fn test_validate_record() {
        let processor = DataProcessor::new(',', false);
        
        let valid_record = vec!["data".to_string(), "value".to_string()];
        assert!(processor.validate_record(&valid_record));

        let invalid_record = vec!["".to_string(), "value".to_string()];
        assert!(!processor.validate_record(&invalid_record));
    }

    #[test]
    fn test_extract_column() {
        let data = vec![
            vec!["a".to_string(), "b".to_string(), "c".to_string()],
            vec!["d".to_string(), "e".to_string(), "f".to_string()],
        ];

        let processor = DataProcessor::new(',', false);
        let column = processor.extract_column(&data, 1);

        assert_eq!(column, vec!["b".to_string(), "e".to_string()]);
    }
}
use std::collections::HashMap;

pub struct DataProcessor {
    data: HashMap<String, Vec<f64>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            data: HashMap::new(),
        }
    }

    pub fn add_dataset(&mut self, key: &str, values: Vec<f64>) -> Result<(), String> {
        if values.is_empty() {
            return Err("Dataset cannot be empty".to_string());
        }

        if values.iter().any(|&x| x.is_nan() || x.is_infinite()) {
            return Err("Dataset contains invalid numeric values".to_string());
        }

        self.data.insert(key.to_string(), values);
        Ok(())
    }

    pub fn calculate_statistics(&self, key: &str) -> Option<Statistics> {
        self.data.get(key).map(|values| {
            let count = values.len();
            let sum: f64 = values.iter().sum();
            let mean = sum / count as f64;
            
            let variance: f64 = values.iter()
                .map(|&x| (x - mean).powi(2))
                .sum::<f64>() / count as f64;
            
            let std_dev = variance.sqrt();

            Statistics {
                count,
                mean,
                variance,
                std_dev,
            }
        })
    }

    pub fn normalize_data(&self, key: &str) -> Option<Vec<f64>> {
        self.data.get(key).map(|values| {
            let stats = self.calculate_statistics(key).unwrap();
            values.iter()
                .map(|&x| (x - stats.mean) / stats.std_dev)
                .collect()
        })
    }

    pub fn merge_datasets(&mut self, key1: &str, key2: &str, new_key: &str) -> Result<(), String> {
        let data1 = self.data.get(key1).ok_or("First dataset not found")?;
        let data2 = self.data.get(key2).ok_or("Second dataset not found")?;

        let mut merged = data1.clone();
        merged.extend(data2.iter().cloned());
        
        self.add_dataset(new_key, merged)
    }
}

pub struct Statistics {
    pub count: usize,
    pub mean: f64,
    pub variance: f64,
    pub std_dev: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_valid_dataset() {
        let mut processor = DataProcessor::new();
        let result = processor.add_dataset("test", vec![1.0, 2.0, 3.0]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_add_invalid_dataset() {
        let mut processor = DataProcessor::new();
        let result = processor.add_dataset("test", vec![]);
        assert!(result.is_err());
    }

    #[test]
    fn test_calculate_statistics() {
        let mut processor = DataProcessor::new();
        processor.add_dataset("test", vec![1.0, 2.0, 3.0]).unwrap();
        
        let stats = processor.calculate_statistics("test").unwrap();
        assert_eq!(stats.count, 3);
        assert!((stats.mean - 2.0).abs() < 1e-10);
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct DataRecord {
    id: u32,
    value: f64,
    category: String,
    valid: bool,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: String) -> Self {
        let valid = value >= 0.0 && value <= 1000.0;
        DataRecord {
            id,
            value,
            category,
            valid,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.valid
    }

    pub fn get_summary(&self) -> String {
        format!("ID: {}, Value: {:.2}, Category: {}", self.id, self.value, self.category)
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
            self.records.push(record);
            count += 1;
        }

        Ok(count)
    }

    pub fn filter_valid(&self) -> Vec<&DataRecord> {
        self.records.iter().filter(|r| r.is_valid()).collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        let valid_records: Vec<&DataRecord> = self.filter_valid();
        
        if valid_records.is_empty() {
            return None;
        }

        let sum: f64 = valid_records.iter().map(|r| r.value).sum();
        Some(sum / valid_records.len() as f64)
    }

    pub fn get_statistics(&self) -> (usize, usize, Option<f64>) {
        let total = self.records.len();
        let valid_count = self.filter_valid().len();
        let average = self.calculate_average();
        
        (total, valid_count, average)
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
        assert_eq!(record.id, 1);
        assert_eq!(record.value, 42.5);
        assert_eq!(record.category, "test");
        assert!(record.is_valid());
    }

    #[test]
    fn test_invalid_record() {
        let record = DataRecord::new(2, -10.0, "test".to_string());
        assert!(!record.is_valid());
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
        assert_eq!(result.unwrap(), 2);
        assert_eq!(processor.records.len(), 2);
    }

    #[test]
    fn test_statistics() {
        let mut processor = DataProcessor::new();
        processor.records.push(DataRecord::new(1, 100.0, "a".to_string()));
        processor.records.push(DataRecord::new(2, 200.0, "b".to_string()));
        processor.records.push(DataRecord::new(3, -50.0, "c".to_string()));

        let (total, valid, avg) = processor.get_statistics();
        
        assert_eq!(total, 3);
        assert_eq!(valid, 2);
        assert_eq!(avg, Some(150.0));
    }
}