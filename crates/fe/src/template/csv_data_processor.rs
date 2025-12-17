
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use csv::{ReaderBuilder, WriterBuilder};

#[derive(Debug, Clone)]
pub struct DataRecord {
    id: u32,
    category: String,
    value: f64,
    active: bool,
}

impl DataRecord {
    pub fn new(id: u32, category: String, value: f64, active: bool) -> Self {
        DataRecord {
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
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut csv_reader = ReaderBuilder::new()
            .has_headers(true)
            .from_reader(reader);

        for result in csv_reader.deserialize() {
            let record: DataRecord = result?;
            self.records.push(record);
        }

        Ok(())
    }

    pub fn save_to_csv(&self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::create(file_path)?;
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

    pub fn filter_by_category(&self, category: &str) -> Vec<DataRecord> {
        self.records
            .iter()
            .filter(|r| r.category == category)
            .cloned()
            .collect()
    }

    pub fn filter_active(&self) -> Vec<DataRecord> {
        self.records
            .iter()
            .filter(|r| r.active)
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

    pub fn calculate_category_average(&self, category: &str) -> f64 {
        let filtered: Vec<&DataRecord> = self.records
            .iter()
            .filter(|r| r.category == category)
            .collect();

        if filtered.is_empty() {
            return 0.0;
        }

        let sum: f64 = filtered.iter().map(|r| r.value).sum();
        sum / filtered.len() as f64
    }

    pub fn add_record(&mut self, record: DataRecord) {
        self.records.push(record);
    }

    pub fn remove_record(&mut self, id: u32) -> bool {
        let original_len = self.records.len();
        self.records.retain(|r| r.id != id);
        self.records.len() < original_len
    }

    pub fn get_record_count(&self) -> usize {
        self.records.len()
    }

    pub fn clear(&mut self) {
        self.records.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_processor_operations() {
        let mut processor = DataProcessor::new();
        
        assert_eq!(processor.get_record_count(), 0);
        
        processor.add_record(DataRecord::new(1, "A".to_string(), 10.5, true));
        processor.add_record(DataRecord::new(2, "B".to_string(), 20.0, false));
        processor.add_record(DataRecord::new(3, "A".to_string(), 30.5, true));
        
        assert_eq!(processor.get_record_count(), 3);
        assert_eq!(processor.calculate_average(), 20.333333333333332);
        assert_eq!(processor.calculate_category_average("A"), 20.5);
        
        let active_records = processor.filter_active();
        assert_eq!(active_records.len(), 2);
        
        let category_a_records = processor.filter_by_category("A");
        assert_eq!(category_a_records.len(), 2);
        
        assert!(processor.remove_record(2));
        assert_eq!(processor.get_record_count(), 2);
        
        processor.clear();
        assert_eq!(processor.get_record_count(), 0);
    }
}