
use csv::{ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
pub struct Record {
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

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let mut rdr = ReaderBuilder::new()
            .has_headers(true)
            .from_path(path)?;

        for result in rdr.deserialize() {
            let record: Record = result?;
            self.records.push(record);
        }

        Ok(())
    }

    pub fn validate_records(&self) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|r| r.value >= 0.0 && !r.name.is_empty())
            .collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        let valid_records = self.validate_records();
        if valid_records.is_empty() {
            return None;
        }

        let sum: f64 = valid_records.iter().map(|r| r.value).sum();
        Some(sum / valid_records.len() as f64)
    }

    pub fn export_validated<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn Error>> {
        let valid_records = self.validate_records();
        let mut wtr = WriterBuilder::new().from_path(path)?;

        for record in valid_records {
            wtr.serialize(record)?;
        }

        wtr.flush()?;
        Ok(())
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|r| r.category == category)
            .collect()
    }
}

impl Default for DataProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processor_creation() {
        let processor = DataProcessor::new();
        assert!(processor.records.is_empty());
    }

    #[test]
    fn test_validate_records() {
        let mut processor = DataProcessor::new();
        processor.records.push(Record {
            id: 1,
            name: "Test".to_string(),
            value: 10.5,
            category: "A".to_string(),
        });

        let valid = processor.validate_records();
        assert_eq!(valid.len(), 1);
    }

    #[test]
    fn test_calculate_average() {
        let mut processor = DataProcessor::new();
        processor.records.push(Record {
            id: 1,
            name: "Item1".to_string(),
            value: 10.0,
            category: "B".to_string(),
        });
        processor.records.push(Record {
            id: 2,
            name: "Item2".to_string(),
            value: 20.0,
            category: "B".to_string(),
        });

        let avg = processor.calculate_average();
        assert_eq!(avg, Some(15.0));
    }
}