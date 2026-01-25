
use csv::{ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub category: String,
    pub active: bool,
}

impl DataRecord {
    pub fn new(id: u32, name: String, value: f64, category: String, active: bool) -> Self {
        Self {
            id,
            name,
            value,
            category,
            active,
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.name.is_empty() && self.value >= 0.0 && !self.category.is_empty()
    }

    pub fn format_summary(&self) -> String {
        format!("ID: {}, Name: {}, Value: {:.2}", self.id, self.name, self.value)
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

    pub fn add_record(&mut self, record: DataRecord) {
        if record.is_valid() {
            self.records.push(record);
        }
    }

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
        let mut rdr = ReaderBuilder::new().has_headers(true).from_reader(file);

        for result in rdr.deserialize() {
            let record: DataRecord = result?;
            if record.is_valid() {
                self.records.push(record);
            }
        }

        Ok(())
    }

    pub fn save_to_csv<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn Error>> {
        let file = File::create(path)?;
        let mut wtr = WriterBuilder::new().has_headers(true).from_writer(file);

        for record in &self.records {
            wtr.serialize(record)?;
        }

        wtr.flush()?;
        Ok(())
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<DataRecord> {
        self.records
            .iter()
            .filter(|r| r.category == category)
            .cloned()
            .collect()
    }

    pub fn calculate_total_value(&self) -> f64 {
        self.records.iter().map(|r| r.value).sum()
    }

    pub fn get_active_records(&self) -> Vec<&DataRecord> {
        self.records.iter().filter(|r| r.active).collect()
    }

    pub fn find_by_id(&self, id: u32) -> Option<&DataRecord> {
        self.records.iter().find(|r| r.id == id)
    }

    pub fn count_records(&self) -> usize {
        self.records.len()
    }

    pub fn clear(&mut self) {
        self.records.clear();
    }

    pub fn get_records(&self) -> &[DataRecord] {
        &self.records
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_record_validation() {
        let valid_record = DataRecord::new(1, "Test".to_string(), 10.5, "A".to_string(), true);
        assert!(valid_record.is_valid());

        let invalid_record = DataRecord::new(2, "".to_string(), -5.0, "".to_string(), false);
        assert!(!invalid_record.is_valid());
    }

    #[test]
    fn test_data_processor_operations() {
        let mut processor = DataProcessor::new();
        
        let record1 = DataRecord::new(1, "Item1".to_string(), 100.0, "CategoryA".to_string(), true);
        let record2 = DataRecord::new(2, "Item2".to_string(), 200.0, "CategoryB".to_string(), false);
        
        processor.add_record(record1.clone());
        processor.add_record(record2.clone());
        
        assert_eq!(processor.count_records(), 2);
        assert_eq!(processor.calculate_total_value(), 300.0);
        
        let active_records = processor.get_active_records();
        assert_eq!(active_records.len(), 1);
        assert_eq!(active_records[0].id, 1);
        
        let found = processor.find_by_id(1);
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "Item1");
    }

    #[test]
    fn test_csv_operations() -> Result<(), Box<dyn Error>> {
        let mut processor = DataProcessor::new();
        let record = DataRecord::new(1, "TestItem".to_string(), 50.0, "TestCat".to_string(), true);
        processor.add_record(record);
        
        let temp_file = NamedTempFile::new()?;
        let path = temp_file.path();
        
        processor.save_to_csv(path)?;
        
        let mut new_processor = DataProcessor::new();
        new_processor.load_from_csv(path)?;
        
        assert_eq!(new_processor.count_records(), 1);
        assert_eq!(new_processor.calculate_total_value(), 50.0);
        
        Ok(())
    }
}