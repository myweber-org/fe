
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub category: String,
    pub valid: bool,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: &str) -> Self {
        let valid = value >= 0.0 && value <= 1000.0;
        DataRecord {
            id,
            value,
            category: category.to_string(),
            valid,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.valid
    }

    pub fn summary(&self) -> String {
        format!(
            "Record {}: {} ({}) - {}",
            self.id,
            self.value,
            self.category,
            if self.valid { "Valid" } else { "Invalid" }
        )
    }
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
    valid_count: usize,
    invalid_count: usize,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
            valid_count: 0,
            invalid_count: 0,
        }
    }

    pub fn load_from_csv(&mut self, file_path: &Path) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut line_count = 0;

        for line in reader.lines().skip(1) {
            let line = line?;
            let parts: Vec<&str> = line.split(',').collect();
            
            if parts.len() >= 3 {
                let id = parts[0].parse::<u32>().unwrap_or(0);
                let value = parts[1].parse::<f64>().unwrap_or(0.0);
                let category = parts[2];
                
                let record = DataRecord::new(id, value, category);
                self.add_record(record);
                line_count += 1;
            }
        }

        println!("Loaded {} records from CSV", line_count);
        Ok(())
    }

    pub fn add_record(&mut self, record: DataRecord) {
        if record.is_valid() {
            self.valid_count += 1;
        } else {
            self.invalid_count += 1;
        }
        self.records.push(record);
    }

    pub fn get_valid_records(&self) -> Vec<&DataRecord> {
        self.records.iter().filter(|r| r.is_valid()).collect()
    }

    pub fn get_invalid_records(&self) -> Vec<&DataRecord> {
        self.records.iter().filter(|r| !r.is_valid()).collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        let valid_records = self.get_valid_records();
        if valid_records.is_empty() {
            return None;
        }

        let sum: f64 = valid_records.iter().map(|r| r.value).sum();
        Some(sum / valid_records.len() as f64)
    }

    pub fn generate_report(&self) -> String {
        let avg = self.calculate_average().unwrap_or(0.0);
        format!(
            "Data Processing Report\n\
            Total Records: {}\n\
            Valid Records: {}\n\
            Invalid Records: {}\n\
            Average Value: {:.2}\n\
            Processing Complete",
            self.records.len(),
            self.valid_count,
            self.invalid_count,
            avg
        )
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|r| r.category == category && r.is_valid())
            .collect()
    }

    pub fn clear(&mut self) {
        self.records.clear();
        self.valid_count = 0;
        self.invalid_count = 0;
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
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_record_creation() {
        let record = DataRecord::new(1, 42.5, "A");
        assert_eq!(record.id, 1);
        assert_eq!(record.value, 42.5);
        assert_eq!(record.category, "A");
        assert!(record.is_valid());
    }

    #[test]
    fn test_invalid_record() {
        let record = DataRecord::new(2, -10.0, "B");
        assert!(!record.is_valid());
    }

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        let record1 = DataRecord::new(1, 100.0, "Test");
        let record2 = DataRecord::new(2, 1500.0, "Test");
        
        processor.add_record(record1);
        processor.add_record(record2);
        
        assert_eq!(processor.valid_count, 1);
        assert_eq!(processor.invalid_count, 1);
        assert_eq!(processor.get_valid_records().len(), 1);
    }

    #[test]
    fn test_csv_loading() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category").unwrap();
        writeln!(temp_file, "1,100.5,CategoryA").unwrap();
        writeln!(temp_file, "2,200.3,CategoryB").unwrap();
        writeln!(temp_file, "3,-50.0,CategoryC").unwrap();
        
        let mut processor = DataProcessor::new();
        let result = processor.load_from_csv(temp_file.path());
        
        assert!(result.is_ok());
        assert_eq!(processor.records.len(), 3);
        assert_eq!(processor.valid_count, 2);
        assert_eq!(processor.invalid_count, 1);
    }

    #[test]
    fn test_average_calculation() {
        let mut processor = DataProcessor::new();
        processor.add_record(DataRecord::new(1, 100.0, "A"));
        processor.add_record(DataRecord::new(2, 200.0, "A"));
        processor.add_record(DataRecord::new(3, 300.0, "B"));
        
        let avg = processor.calculate_average();
        assert_eq!(avg, Some(200.0));
    }

    #[test]
    fn test_filter_by_category() {
        let mut processor = DataProcessor::new();
        processor.add_record(DataRecord::new(1, 100.0, "A"));
        processor.add_record(DataRecord::new(2, 200.0, "B"));
        processor.add_record(DataRecord::new(3, 300.0, "A"));
        
        let filtered = processor.filter_by_category("A");
        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().all(|r| r.category == "A"));
    }
}