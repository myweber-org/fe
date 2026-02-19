use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug)]
pub struct Record {
    id: u32,
    category: String,
    value: f64,
    active: bool,
}

impl Record {
    pub fn new(id: u32, category: String, value: f64, active: bool) -> Self {
        Record {
            id,
            category,
            value,
            active,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.value > 0.0 && !self.category.is_empty()
    }
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

    pub fn load_from_file(&mut self, path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        
        for (index, line) in reader.lines().enumerate() {
            let line = line?;
            if index == 0 {
                continue;
            }
            
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() >= 4 {
                let id = parts[0].parse::<u32>().unwrap_or(0);
                let category = parts[1].to_string();
                let value = parts[2].parse::<f64>().unwrap_or(0.0);
                let active = parts[3].parse::<bool>().unwrap_or(false);
                
                self.records.push(Record::new(id, category, value, active));
            }
        }
        
        Ok(())
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|record| record.category == category && record.is_valid())
            .collect()
    }

    pub fn calculate_average(&self) -> f64 {
        let valid_records: Vec<&Record> = self.records
            .iter()
            .filter(|record| record.is_valid())
            .collect();
        
        if valid_records.is_empty() {
            return 0.0;
        }
        
        let total: f64 = valid_records.iter().map(|record| record.value).sum();
        total / valid_records.len() as f64
    }

    pub fn get_active_records(&self) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|record| record.active && record.is_valid())
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
    fn test_record_validation() {
        let valid_record = Record::new(1, "test".to_string(), 10.5, true);
        assert!(valid_record.is_valid());
        
        let invalid_record = Record::new(2, "".to_string(), -5.0, false);
        assert!(!invalid_record.is_valid());
    }

    #[test]
    fn test_data_processing() {
        let mut processor = DataProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,category,value,active").unwrap();
        writeln!(temp_file, "1,electronics,100.0,true").unwrap();
        writeln!(temp_file, "2,books,25.5,false").unwrap();
        writeln!(temp_file, "3,electronics,75.0,true").unwrap();
        
        let result = processor.load_from_file(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(processor.count_records(), 3);
        
        let electronics = processor.filter_by_category("electronics");
        assert_eq!(electronics.len(), 2);
        
        let average = processor.calculate_average();
        assert!(average > 0.0);
        
        let active = processor.get_active_records();
        assert_eq!(active.len(), 2);
    }
}