
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, PartialEq)]
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

pub struct CsvProcessor {
    records: Vec<Record>,
}

impl CsvProcessor {
    pub fn new() -> Self {
        CsvProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        
        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            if line_num == 0 {
                continue;
            }
            
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() != 4 {
                return Err(format!("Invalid CSV format at line {}", line_num + 1).into());
            }
            
            let id = parts[0].parse::<u32>()?;
            let name = parts[1].to_string();
            let value = parts[2].parse::<f64>()?;
            let active = parts[3].parse::<bool>()?;
            
            let record = Record::new(id, name, value, active);
            record.validate()?;
            self.records.push(record);
        }
        
        Ok(())
    }

    pub fn filter_active(&self) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|r| r.active)
            .collect()
    }

    pub fn calculate_total(&self) -> f64 {
        self.records
            .iter()
            .map(|r| r.value)
            .sum()
    }

    pub fn find_by_id(&self, target_id: u32) -> Option<&Record> {
        self.records
            .iter()
            .find(|r| r.id == target_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_record_validation() {
        let valid_record = Record::new(1, "Test".to_string(), 100.0, true);
        assert!(valid_record.validate().is_ok());

        let invalid_record = Record::new(2, "".to_string(), -50.0, false);
        assert!(invalid_record.validate().is_err());
    }

    #[test]
    fn test_csv_processing() {
        let mut csv_data = NamedTempFile::new().unwrap();
        writeln!(csv_data, "id,name,value,active").unwrap();
        writeln!(csv_data, "1,Alice,42.5,true").unwrap();
        writeln!(csv_data, "2,Bob,73.2,false").unwrap();
        writeln!(csv_data, "3,Charlie,15.8,true").unwrap();

        let mut processor = CsvProcessor::new();
        processor.load_from_file(csv_data.path()).unwrap();

        assert_eq!(processor.records.len(), 3);
        assert_eq!(processor.filter_active().len(), 2);
        assert!((processor.calculate_total() - 131.5).abs() < 0.001);
        assert!(processor.find_by_id(2).is_some());
        assert!(processor.find_by_id(99).is_none());
    }
}