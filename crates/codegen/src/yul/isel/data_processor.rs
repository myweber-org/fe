
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;

use serde::{Deserialize, Serialize};
use csv::{ReaderBuilder, WriterBuilder};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Record {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub category: String,
}

impl Record {
    pub fn is_valid(&self) -> bool {
        !self.name.is_empty() && 
        self.value >= 0.0 && 
        !self.category.is_empty()
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
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        
        let mut csv_reader = ReaderBuilder::new()
            .has_headers(true)
            .from_reader(reader);

        for result in csv_reader.deserialize() {
            let record: Record = result?;
            if record.is_valid() {
                self.records.push(record);
            }
        }

        Ok(())
    }

    pub fn save_to_csv<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn Error>> {
        let file = File::create(path)?;
        let writer = BufWriter::new(file);
        
        let mut csv_writer = WriterBuilder::new()
            .has_headers(true)
            .from_writer(writer);

        for record in &self.records {
            csv_writer.serialize(record)?;
        }

        csv_writer.flush()?;
        Ok(())
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<Record> {
        self.records
            .iter()
            .filter(|r| r.category == category)
            .cloned()
            .collect()
    }

    pub fn calculate_average(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }
        
        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        sum / self.records.len() as f64
    }

    pub fn add_record(&mut self, record: Record) {
        if record.is_valid() {
            self.records.push(record);
        }
    }

    pub fn get_records(&self) -> &[Record] {
        &self.records
    }

    pub fn clear(&mut self) {
        self.records.clear();
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
            value: 10.5,
            category: "A".to_string(),
        };
        assert!(valid_record.is_valid());

        let invalid_record = Record {
            id: 2,
            name: "".to_string(),
            value: -5.0,
            category: "B".to_string(),
        };
        assert!(!invalid_record.is_valid());
    }

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        
        let record = Record {
            id: 1,
            name: "Item1".to_string(),
            value: 100.0,
            category: "Electronics".to_string(),
        };
        
        processor.add_record(record);
        assert_eq!(processor.get_records().len(), 1);
        assert_eq!(processor.calculate_average(), 100.0);
    }

    #[test]
    fn test_csv_operations() -> Result<(), Box<dyn Error>> {
        let mut processor = DataProcessor::new();
        
        let record1 = Record {
            id: 1,
            name: "Item1".to_string(),
            value: 50.0,
            category: "CategoryA".to_string(),
        };
        
        let record2 = Record {
            id: 2,
            name: "Item2".to_string(),
            value: 75.0,
            category: "CategoryB".to_string(),
        };
        
        processor.add_record(record1);
        processor.add_record(record2);

        let temp_file = NamedTempFile::new()?;
        let path = temp_file.path();
        
        processor.save_to_csv(path)?;
        
        let mut new_processor = DataProcessor::new();
        new_processor.load_from_csv(path)?;
        
        assert_eq!(new_processor.get_records().len(), 2);
        assert_eq!(new_processor.calculate_average(), 62.5);
        
        Ok(())
    }
}