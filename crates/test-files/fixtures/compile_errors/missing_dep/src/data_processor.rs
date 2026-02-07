
use csv::Reader;
use serde::Deserialize;
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    active: bool,
}

pub fn process_csv_file(file_path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let mut reader = Reader::from_reader(file);
    let mut records = Vec::new();

    for result in reader.deserialize() {
        let record: Record = result?;
        if record.value >= 0.0 {
            records.push(record);
        }
    }

    Ok(records)
}

pub fn calculate_statistics(records: &[Record]) -> (f64, f64, usize) {
    let sum: f64 = records.iter().map(|r| r.value).sum();
    let count = records.len();
    let average = if count > 0 { sum / count as f64 } else { 0.0 };
    
    let active_count = records.iter().filter(|r| r.active).count();
    
    (sum, average, active_count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_process_valid_csv() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,active").unwrap();
        writeln!(temp_file, "1,Test1,10.5,true").unwrap();
        writeln!(temp_file, "2,Test2,-3.0,false").unwrap();
        writeln!(temp_file, "3,Test3,7.2,true").unwrap();

        let records = process_csv_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].name, "Test1");
        assert_eq!(records[1].name, "Test3");
    }

    #[test]
    fn test_calculate_statistics() {
        let records = vec![
            Record { id: 1, name: "A".to_string(), value: 10.0, active: true },
            Record { id: 2, name: "B".to_string(), value: 20.0, active: false },
            Record { id: 3, name: "C".to_string(), value: 30.0, active: true },
        ];

        let (sum, avg, active_count) = calculate_statistics(&records);
        assert_eq!(sum, 60.0);
        assert_eq!(avg, 20.0);
        assert_eq!(active_count, 2);
    }
}
use csv::{ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
pub struct DataRecord {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

impl DataRecord {
    pub fn new(id: u32, name: String, value: f64, category: String) -> Self {
        Self {
            id,
            name,
            value,
            category,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("Name cannot be empty".to_string());
        }
        if self.value < 0.0 {
            return Err("Value must be non-negative".to_string());
        }
        if self.category.is_empty() {
            return Err("Category cannot be empty".to_string());
        }
        Ok(())
    }
}

pub struct DataProcessor;

impl DataProcessor {
    pub fn load_from_csv<P: AsRef<Path>>(path: P) -> Result<Vec<DataRecord>, Box<dyn Error>> {
        let mut reader = ReaderBuilder::new()
            .has_headers(true)
            .from_path(path)?;

        let mut records = Vec::new();
        for result in reader.deserialize() {
            let record: DataRecord = result?;
            record.validate()?;
            records.push(record);
        }

        Ok(records)
    }

    pub fn save_to_csv<P: AsRef<Path>>(
        records: &[DataRecord],
        path: P,
    ) -> Result<(), Box<dyn Error>> {
        let mut writer = WriterBuilder::new()
            .has_headers(true)
            .from_path(path)?;

        for record in records {
            record.validate()?;
            writer.serialize(record)?;
        }

        writer.flush()?;
        Ok(())
    }

    pub fn filter_by_category(records: &[DataRecord], category: &str) -> Vec<DataRecord> {
        records
            .iter()
            .filter(|r| r.category == category)
            .cloned()
            .collect()
    }

    pub fn calculate_average(records: &[DataRecord]) -> Option<f64> {
        if records.is_empty() {
            return None;
        }

        let sum: f64 = records.iter().map(|r| r.value).sum();
        Some(sum / records.len() as f64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_record_validation() {
        let valid_record = DataRecord::new(1, "Test".to_string(), 42.5, "A".to_string());
        assert!(valid_record.validate().is_ok());

        let invalid_record = DataRecord::new(2, "".to_string(), -1.0, "".to_string());
        assert!(invalid_record.validate().is_err());
    }

    #[test]
    fn test_csv_operations() -> Result<(), Box<dyn Error>> {
        let records = vec![
            DataRecord::new(1, "Alpha".to_string(), 10.5, "X".to_string()),
            DataRecord::new(2, "Beta".to_string(), 20.3, "Y".to_string()),
            DataRecord::new(3, "Gamma".to_string(), 15.7, "X".to_string()),
        ];

        let temp_file = NamedTempFile::new()?;
        let path = temp_file.path();

        DataProcessor::save_to_csv(&records, path)?;
        let loaded = DataProcessor::load_from_csv(path)?;

        assert_eq!(records.len(), loaded.len());
        Ok(())
    }

    #[test]
    fn test_filter_and_average() {
        let records = vec![
            DataRecord::new(1, "A".to_string(), 10.0, "Cat1".to_string()),
            DataRecord::new(2, "B".to_string(), 20.0, "Cat2".to_string()),
            DataRecord::new(3, "C".to_string(), 30.0, "Cat1".to_string()),
        ];

        let filtered = DataProcessor::filter_by_category(&records, "Cat1");
        assert_eq!(filtered.len(), 2);

        let avg = DataProcessor::calculate_average(&filtered);
        assert_eq!(avg, Some(20.0));
    }
}