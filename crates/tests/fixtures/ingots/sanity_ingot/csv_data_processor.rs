use std::error::Error;
use std::fs::File;
use csv::{Reader, Writer};

#[derive(Debug, Clone)]
pub struct DataRecord {
    id: u32,
    category: String,
    value: f64,
    active: bool,
}

impl DataRecord {
    pub fn new(id: u32, category: String, value: f64, active: bool) -> Self {
        Self {
            id,
            category,
            value,
            active,
        }
    }
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
}

impl DataProcessor {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn Error>> {
        let file = File::open(path)?;
        let mut rdr = Reader::from_reader(file);
        let mut records = Vec::new();

        for result in rdr.deserialize() {
            let record: DataRecord = result?;
            records.push(record);
        }

        Ok(Self { records })
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<DataRecord> {
        self.records
            .iter()
            .filter(|r| r.category == category)
            .cloned()
            .collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }

        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn export_active_records(&self, output_path: &str) -> Result<(), Box<dyn Error>> {
        let active_records: Vec<&DataRecord> = self.records.iter().filter(|r| r.active).collect();
        
        let mut wtr = Writer::from_path(output_path)?;
        
        for record in active_records {
            wtr.serialize(record)?;
        }
        
        wtr.flush()?;
        Ok(())
    }

    pub fn add_record(&mut self, record: DataRecord) {
        self.records.push(record);
    }

    pub fn total_records(&self) -> usize {
        self.records.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processor_operations() {
        let mut processor = DataProcessor { records: Vec::new() };
        
        processor.add_record(DataRecord::new(1, "A".to_string(), 10.5, true));
        processor.add_record(DataRecord::new(2, "B".to_string(), 20.0, false));
        processor.add_record(DataRecord::new(3, "A".to_string(), 15.5, true));
        
        assert_eq!(processor.total_records(), 3);
        
        let filtered = processor.filter_by_category("A");
        assert_eq!(filtered.len(), 2);
        
        let avg = processor.calculate_average();
        assert!(avg.is_some());
        assert!((avg.unwrap() - 15.333).abs() < 0.001);
    }

    #[test]
    fn test_export_functionality() {
        let mut processor = DataProcessor { records: Vec::new() };
        processor.add_record(DataRecord::new(1, "Test".to_string(), 100.0, true));
        
        let temp_file = NamedTempFile::new().unwrap();
        let result = processor.export_active_records(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
    }
}