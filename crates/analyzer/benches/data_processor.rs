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
        let mut reader = ReaderBuilder::new()
            .has_headers(true)
            .delimiter(b',')
            .from_path(path)?;

        for result in reader.deserialize() {
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
        if self.records.is_empty() {
            return None;
        }

        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn export_valid_records<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn Error>> {
        let valid_records = self.validate_records();
        
        let mut writer = WriterBuilder::new()
            .has_headers(true)
            .delimiter(b',')
            .from_path(path)?;

        for record in valid_records {
            writer.serialize(record)?;
        }

        writer.flush()?;
        Ok(())
    }

    pub fn group_by_category(&self) -> std::collections::HashMap<String, Vec<&Record>> {
        let mut groups = std::collections::HashMap::new();
        
        for record in &self.records {
            groups
                .entry(record.category.clone())
                .or_insert_with(Vec::new)
                .push(record);
        }
        
        groups
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_empty_processor() {
        let processor = DataProcessor::new();
        assert_eq!(processor.calculate_average(), None);
    }

    #[test]
    fn test_record_validation() {
        let mut processor = DataProcessor::new();
        processor.records.push(Record {
            id: 1,
            name: "Test".to_string(),
            value: 10.5,
            category: "A".to_string(),
        });
        
        processor.records.push(Record {
            id: 2,
            name: "".to_string(),
            value: -5.0,
            category: "B".to_string(),
        });

        let valid = processor.validate_records();
        assert_eq!(valid.len(), 1);
    }

    #[test]
    fn test_export_functionality() -> Result<(), Box<dyn Error>> {
        let mut processor = DataProcessor::new();
        processor.records.push(Record {
            id: 1,
            name: "ExportTest".to_string(),
            value: 42.0,
            category: "Test".to_string(),
        });

        let temp_file = NamedTempFile::new()?;
        processor.export_valid_records(temp_file.path())?;
        
        Ok(())
    }
}