
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

pub struct DataProcessor {
    records: Vec<Record>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let mut rdr = Reader::from_path(path)?;
        for result in rdr.deserialize() {
            let record: Record = result?;
            record.validate()?;
            self.records.push(record);
        }
        Ok(())
    }

    pub fn save_to_csv<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn Error>> {
        let mut wtr = Writer::from_path(path)?;
        for record in &self.records {
            wtr.serialize(record)?;
        }
        wtr.flush()?;
        Ok(())
    }

    pub fn filter_active(&self) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|record| record.active)
            .collect()
    }

    pub fn calculate_total(&self) -> f64 {
        self.records.iter().map(|record| record.value).sum()
    }

    pub fn add_record(&mut self, record: Record) -> Result<(), String> {
        record.validate()?;
        self.records.push(record);
        Ok(())
    }

    pub fn get_record_count(&self) -> usize {
        self.records.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_record_validation() {
        let valid_record = Record {
            id: 1,
            name: "Test".to_string(),
            value: 100.0,
            active: true,
        };
        assert!(valid_record.validate().is_ok());

        let invalid_record = Record {
            id: 2,
            name: "".to_string(),
            value: -50.0,
            active: false,
        };
        assert!(invalid_record.validate().is_err());
    }

    #[test]
    fn test_data_processor_operations() {
        let mut processor = DataProcessor::new();
        
        let record = Record {
            id: 1,
            name: "Sample".to_string(),
            value: 42.5,
            active: true,
        };
        
        assert!(processor.add_record(record).is_ok());
        assert_eq!(processor.get_record_count(), 1);
        assert_eq!(processor.calculate_total(), 42.5);
        
        let active_records = processor.filter_active();
        assert_eq!(active_records.len(), 1);
    }

    #[test]
    fn test_csv_roundtrip() {
        let mut processor = DataProcessor::new();
        let record = Record {
            id: 99,
            name: "Roundtrip".to_string(),
            value: 3.14,
            active: false,
        };
        
        processor.add_record(record).unwrap();
        
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();
        
        processor.save_to_csv(path).unwrap();
        
        let mut new_processor = DataProcessor::new();
        new_processor.load_from_csv(path).unwrap();
        
        assert_eq!(new_processor.get_record_count(), 1);
        assert_eq!(new_processor.calculate_total(), 3.14);
    }
}