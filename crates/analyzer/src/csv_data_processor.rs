use std::error::Error;
use std::fs::File;
use csv::{ReaderBuilder, WriterBuilder};

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub category: String,
    pub value: f64,
    pub active: bool,
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let mut reader = ReaderBuilder::new()
            .has_headers(true)
            .from_reader(file);

        for result in reader.deserialize() {
            let record: DataRecord = result?;
            self.records.push(record);
        }

        Ok(())
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<DataRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .cloned()
            .collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }

        let sum: f64 = self.records.iter().map(|record| record.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn export_active_records(&self, output_path: &str) -> Result<(), Box<dyn Error>> {
        let active_records: Vec<&DataRecord> = self
            .records
            .iter()
            .filter(|record| record.active)
            .collect();

        let mut writer = WriterBuilder::new()
            .has_headers(true)
            .from_path(output_path)?;

        for record in active_records {
            writer.serialize(record)?;
        }

        writer.flush()?;
        Ok(())
    }

    pub fn add_record(&mut self, record: DataRecord) {
        self.records.push(record);
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
    fn test_data_processor_operations() {
        let mut processor = DataProcessor::new();
        
        processor.add_record(DataRecord {
            id: 1,
            category: "A".to_string(),
            value: 10.5,
            active: true,
        });

        processor.add_record(DataRecord {
            id: 2,
            category: "B".to_string(),
            value: 20.0,
            active: false,
        });

        assert_eq!(processor.get_record_count(), 2);
        
        let category_a = processor.filter_by_category("A");
        assert_eq!(category_a.len(), 1);
        assert_eq!(category_a[0].id, 1);

        let avg = processor.calculate_average();
        assert!(avg.is_some());
        assert!((avg.unwrap() - 15.25).abs() < 0.001);
    }

    #[test]
    fn test_export_functionality() {
        let mut processor = DataProcessor::new();
        
        processor.add_record(DataRecord {
            id: 1,
            category: "Test".to_string(),
            value: 42.0,
            active: true,
        });

        let temp_file = NamedTempFile::new().unwrap();
        let output_path = temp_file.path().to_str().unwrap();
        
        let result = processor.export_active_records(output_path);
        assert!(result.is_ok());
    }
}