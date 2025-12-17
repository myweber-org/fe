
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct DataProcessor {
    file_path: String,
    delimiter: char,
}

impl DataProcessor {
    pub fn new(file_path: &str, delimiter: char) -> Self {
        DataProcessor {
            file_path: file_path.to_string(),
            delimiter,
        }
    }

    pub fn process(&self) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let path = Path::new(&self.file_path);
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        
        let mut records = Vec::new();
        
        for (line_number, line) in reader.lines().enumerate() {
            let line = line?;
            
            if line.trim().is_empty() {
                continue;
            }
            
            let fields: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();
            
            if fields.iter().all(|f| f.is_empty()) {
                continue;
            }
            
            records.push(fields);
            
            if line_number % 1000 == 0 && line_number > 0 {
                println!("Processed {} lines", line_number);
            }
        }
        
        Ok(records)
    }
    
    pub fn validate_records(&self, records: &[Vec<String>]) -> Result<(), Box<dyn Error>> {
        if records.is_empty() {
            return Err("No valid records found".into());
        }
        
        let header_len = records[0].len();
        
        for (idx, record) in records.iter().enumerate() {
            if record.len() != header_len {
                return Err(format!(
                    "Record {} has {} fields, expected {}",
                    idx,
                    record.len(),
                    header_len
                ).into());
            }
            
            for (field_idx, field) in record.iter().enumerate() {
                if field.is_empty() {
                    return Err(format!(
                        "Empty field at record {}, position {}",
                        idx, field_idx
                    ).into());
                }
            }
        }
        
        Ok(())
    }
}

pub fn calculate_statistics(records: &[Vec<String>]) -> Vec<(String, usize, f64)> {
    if records.is_empty() {
        return Vec::new();
    }
    
    let num_columns = records[0].len();
    let mut results = Vec::with_capacity(num_columns);
    
    for col_idx in 0..num_columns {
        let mut numeric_values = Vec::new();
        let column_name = if col_idx == 0 {
            "ID".to_string()
        } else {
            format!("Column_{}", col_idx)
        };
        
        for record in records.iter().skip(1) {
            if let Some(field) = record.get(col_idx) {
                if let Ok(value) = field.parse::<f64>() {
                    numeric_values.push(value);
                }
            }
        }
        
        if !numeric_values.is_empty() {
            let count = numeric_values.len();
            let sum: f64 = numeric_values.iter().sum();
            let average = sum / count as f64;
            
            results.push((column_name, count, average));
        }
    }
    
    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_data_processor() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value").unwrap();
        writeln!(temp_file, "1,item1,10.5").unwrap();
        writeln!(temp_file, "2,item2,20.3").unwrap();
        writeln!(temp_file, "3,item3,15.7").unwrap();
        
        let processor = DataProcessor::new(temp_file.path().to_str().unwrap(), ',');
        let records = processor.process().unwrap();
        
        assert_eq!(records.len(), 4);
        assert_eq!(records[0], vec!["id", "name", "value"]);
        assert_eq!(records[1], vec!["1", "item1", "10.5"]);
        
        processor.validate_records(&records).unwrap();
        
        let stats = calculate_statistics(&records);
        assert_eq!(stats.len(), 3);
    }
    
    #[test]
    fn test_empty_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let processor = DataProcessor::new(temp_file.path().to_str().unwrap(), ',');
        let records = processor.process().unwrap();
        
        assert!(records.is_empty());
        
        let result = processor.validate_records(&records);
        assert!(result.is_err());
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
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: &str) -> Result<Self, &'static str> {
        if value < 0.0 {
            return Err("Value cannot be negative");
        }
        if category.is_empty() {
            return Err("Category cannot be empty");
        }
        
        Ok(DataRecord {
            id,
            value,
            category: category.to_string(),
        })
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
    
    pub fn load_from_csv(&mut self, file_path: &Path) -> Result<usize, Box<dyn Error>> {
        let file = File::open(file_path)?;
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
                Ok(id) => id,
                Err(_) => continue,
            };
            
            let value = match parts[1].parse::<f64>() {
                Ok(value) => value,
                Err(_) => continue,
            };
            
            let category = parts[2].trim();
            
            match DataRecord::new(id, value, category) {
                Ok(record) => {
                    self.records.push(record);
                    count += 1;
                }
                Err(_) => continue,
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
    
    pub fn get_statistics(&self) -> (f64, f64, f64) {
        if self.records.is_empty() {
            return (0.0, 0.0, 0.0);
        }
        
        let values: Vec<f64> = self.records.iter().map(|record| record.value).collect();
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
    fn test_data_record_creation() {
        let record = DataRecord::new(1, 42.5, "test").unwrap();
        assert_eq!(record.id, 1);
        assert_eq!(record.value, 42.5);
        assert_eq!(record.category, "test");
    }
    
    #[test]
    fn test_invalid_data_record() {
        assert!(DataRecord::new(1, -5.0, "test").is_err());
        assert!(DataRecord::new(1, 5.0, "").is_err());
    }
    
    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        assert_eq!(processor.count_records(), 0);
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category").unwrap();
        writeln!(temp_file, "1,10.5,category_a").unwrap();
        writeln!(temp_file, "2,20.0,category_b").unwrap();
        writeln!(temp_file, "3,15.5,category_a").unwrap();
        
        let count = processor.load_from_csv(temp_file.path()).unwrap();
        assert_eq!(count, 3);
        assert_eq!(processor.count_records(), 3);
        
        let filtered = processor.filter_by_category("category_a");
        assert_eq!(filtered.len(), 2);
        
        let avg = processor.calculate_average().unwrap();
        assert!((avg - 15.333).abs() < 0.001);
        
        let (min, max, avg_stat) = processor.get_statistics();
        assert_eq!(min, 10.5);
        assert_eq!(max, 20.0);
        assert!((avg_stat - 15.333).abs() < 0.001);
    }
}