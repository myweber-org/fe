
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
    pub fn new(id: u32, value: f64, category: &str) -> Result<Self, String> {
        if value < 0.0 {
            return Err("Value cannot be negative".to_string());
        }
        if category.trim().is_empty() {
            return Err("Category cannot be empty".to_string());
        }
        
        Ok(Self {
            id,
            value,
            category: category.to_string(),
        })
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
        Self {
            records: Vec::new(),
        }
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
            
            let id = parts[0].parse::<u32>()
                .map_err(|e| format!("Invalid ID at line {}: {}", line_num + 1, e))?;
            
            let value = parts[1].parse::<f64>()
                .map_err(|e| format!("Invalid value at line {}: {}", line_num + 1, e))?;
            
            let record = DataRecord::new(id, value, parts[2])?;
            self.records.push(record);
            count += 1;
        }
        
        Ok(count)
    }
    
    pub fn total_value(&self) -> f64 {
        self.records.iter().map(|r| r.value).sum()
    }
    
    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|r| r.category == category)
            .collect()
    }
    
    pub fn average_value(&self) -> Option<f64> {
        if self.records.is_empty() {
            None
        } else {
            Some(self.total_value() / self.records.len() as f64)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_record_creation() {
        let record = DataRecord::new(1, 100.5, "A").unwrap();
        assert_eq!(record.id, 1);
        assert_eq!(record.value, 100.5);
        assert_eq!(record.category, "A");
    }
    
    #[test]
    fn test_invalid_record() {
        assert!(DataRecord::new(1, -10.0, "A").is_err());
        assert!(DataRecord::new(1, 10.0, "").is_err());
    }
    
    #[test]
    fn test_tax_calculation() {
        let record = DataRecord::new(1, 100.0, "A").unwrap();
        assert_eq!(record.calculate_tax(0.1), 10.0);
    }
    
    #[test]
    fn test_load_from_file() {
        let mut content = NamedTempFile::new().unwrap();
        writeln!(content, "1,100.5,CategoryA").unwrap();
        writeln!(content, "2,200.0,CategoryB").unwrap();
        writeln!(content, "# This is a comment").unwrap();
        writeln!(content, "").unwrap();
        writeln!(content, "3,150.75,CategoryA").unwrap();
        
        let mut processor = DataProcessor::new();
        let result = processor.load_from_file(content.path());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 3);
        assert_eq!(processor.records.len(), 3);
    }
    
    #[test]
    fn test_filter_and_average() {
        let mut processor = DataProcessor::new();
        processor.records.push(DataRecord::new(1, 100.0, "A").unwrap());
        processor.records.push(DataRecord::new(2, 200.0, "B").unwrap());
        processor.records.push(DataRecord::new(3, 300.0, "A").unwrap());
        
        let filtered = processor.filter_by_category("A");
        assert_eq!(filtered.len(), 2);
        
        let average = processor.average_value();
        assert_eq!(average, Some(200.0));
        
        assert_eq!(processor.total_value(), 600.0);
    }
}