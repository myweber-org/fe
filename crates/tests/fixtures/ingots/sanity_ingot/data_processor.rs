use csv::{Reader, Writer};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct Record {
    id: u32,
    name: String,
    value: f64,
    active: bool,
}

pub struct DataProcessor {
    records: Vec<Record>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor { records: Vec::new() }
    }

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let mut reader = Reader::from_path(path)?;
        for result in reader.deserialize() {
            let record: Record = result?;
            self.records.push(record);
        }
        Ok(())
    }

    pub fn save_to_csv<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn Error>> {
        let mut writer = Writer::from_path(path)?;
        for record in &self.records {
            writer.serialize(record)?;
        }
        writer.flush()?;
        Ok(())
    }

    pub fn add_record(&mut self, record: Record) {
        self.records.push(record);
    }

    pub fn filter_active(&self) -> Vec<&Record> {
        self.records.iter().filter(|r| r.active).collect()
    }

    pub fn calculate_total(&self) -> f64 {
        self.records.iter().map(|r| r.value).sum()
    }

    pub fn find_by_id(&self, id: u32) -> Option<&Record> {
        self.records.iter().find(|r| r.id == id)
    }

    pub fn validate_records(&self) -> Vec<String> {
        let mut errors = Vec::new();
        for record in &self.records {
            if record.name.is_empty() {
                errors.push(format!("Record {} has empty name", record.id));
            }
            if record.value < 0.0 {
                errors.push(format!("Record {} has negative value", record.id));
            }
        }
        errors
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        
        let record1 = Record {
            id: 1,
            name: "Test1".to_string(),
            value: 100.5,
            active: true,
        };
        
        let record2 = Record {
            id: 2,
            name: "Test2".to_string(),
            value: 200.0,
            active: false,
        };
        
        processor.add_record(record1);
        processor.add_record(record2);
        
        assert_eq!(processor.filter_active().len(), 1);
        assert_eq!(processor.calculate_total(), 300.5);
        assert!(processor.find_by_id(1).is_some());
        assert!(processor.find_by_id(3).is_none());
    }

    #[test]
    fn test_csv_roundtrip() {
        let mut processor = DataProcessor::new();
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();
        
        let record = Record {
            id: 42,
            name: "CSV Test".to_string(),
            value: 99.9,
            active: true,
        };
        
        processor.add_record(record);
        processor.save_to_csv(path).unwrap();
        
        let mut new_processor = DataProcessor::new();
        new_processor.load_from_csv(path).unwrap();
        
        assert_eq!(new_processor.records.len(), 1);
        assert_eq!(new_processor.records[0].id, 42);
    }
}