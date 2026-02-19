
use std::error::Error;
use std::fs::File;
use std::path::Path;

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

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
        let mut rdr = csv::Reader::from_reader(file);

        for result in rdr.deserialize() {
            let record: DataRecord = result?;
            self.records.push(record);
        }

        Ok(())
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

    pub fn get_active_records(&self) -> Vec<&DataRecord> {
        self.records.iter().filter(|r| r.active).collect()
    }

    pub fn find_max_value(&self) -> Option<&DataRecord> {
        self.records.iter().max_by(|a, b| {
            a.value
                .partial_cmp(&b.value)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    pub fn count_by_category(&self) -> std::collections::HashMap<String, usize> {
        let mut counts = std::collections::HashMap::new();
        
        for record in &self.records {
            *counts.entry(record.category.clone()).or_insert(0) += 1;
        }
        
        counts
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_csv() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "id,category,value,active").unwrap();
        writeln!(file, "1,electronics,250.5,true").unwrap();
        writeln!(file, "2,books,45.0,false").unwrap();
        writeln!(file, "3,electronics,180.0,true").unwrap();
        writeln!(file, "4,clothing,75.25,true").unwrap();
        file
    }

    #[test]
    fn test_load_and_filter() {
        let csv_file = create_test_csv();
        let mut processor = DataProcessor::new();
        
        processor.load_from_csv(csv_file.path()).unwrap();
        
        let electronics = processor.filter_by_category("electronics");
        assert_eq!(electronics.len(), 2);
        
        let avg = processor.calculate_average();
        assert!(avg.is_some());
        assert!((avg.unwrap() - 137.6875).abs() < 0.001);
    }

    #[test]
    fn test_active_records() {
        let csv_file = create_test_csv();
        let mut processor = DataProcessor::new();
        
        processor.load_from_csv(csv_file.path()).unwrap();
        
        let active = processor.get_active_records();
        assert_eq!(active.len(), 3);
    }

    #[test]
    fn test_count_categories() {
        let csv_file = create_test_csv();
        let mut processor = DataProcessor::new();
        
        processor.load_from_csv(csv_file.path()).unwrap();
        
        let counts = processor.count_by_category();
        assert_eq!(counts.get("electronics"), Some(&2));
        assert_eq!(counts.get("books"), Some(&1));
        assert_eq!(counts.get("clothing"), Some(&1));
    }
}use csv::{Reader, Writer};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    category: String,
    value: f64,
    active: bool,
}

fn filter_records_by_category(records: &[Record], category: &str) -> Vec<&Record> {
    records
        .iter()
        .filter(|record| record.category == category && record.active)
        .collect()
}

fn calculate_average_value(records: &[&Record]) -> Option<f64> {
    if records.is_empty() {
        return None;
    }
    
    let sum: f64 = records.iter().map(|r| r.value).sum();
    Some(sum / records.len() as f64)
}

fn process_csv_file(input_path: &str, output_path: &str, target_category: &str) -> Result<(), Box<dyn Error>> {
    let file = File::open(input_path)?;
    let mut reader = Reader::from_reader(file);
    
    let mut records: Vec<Record> = Vec::new();
    for result in reader.deserialize() {
        let record: Record = result?;
        records.push(record);
    }
    
    let filtered_records = filter_records_by_category(&records, target_category);
    
    match calculate_average_value(&filtered_records) {
        Some(avg) => println!("Average value for category '{}': {:.2}", target_category, avg),
        None => println!("No active records found for category '{}'", target_category),
    }
    
    let output_file = File::create(output_path)?;
    let mut writer = Writer::from_writer(output_file);
    
    for record in filtered_records {
        writer.serialize(record)?;
    }
    
    writer.flush()?;
    println!("Filtered data written to: {}", output_path);
    
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let input_file = "data/input.csv";
    let output_file = "data/output.csv";
    let target_category = "electronics";
    
    process_csv_file(input_file, output_file, target_category)
}
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
        let mut rdr = ReaderBuilder::new()
            .has_headers(true)
            .from_reader(file);

        for result in rdr.deserialize() {
            let record: DataRecord = result?;
            self.records.push(record);
        }

        Ok(())
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
        let active_records: Vec<&DataRecord> = 
            self.records.iter().filter(|r| r.active).collect();

        let mut wtr = WriterBuilder::new()
            .has_headers(true)
            .from_path(output_path)?;

        for record in active_records {
            wtr.serialize(record)?;
        }

        wtr.flush()?;
        Ok(())
    }

    pub fn find_max_value(&self) -> Option<&DataRecord> {
        self.records.iter().max_by(|a, b| {
            a.value.partial_cmp(&b.value).unwrap()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_filter_by_category() {
        let mut processor = DataProcessor::new();
        processor.records = vec![
            DataRecord { id: 1, category: "A".to_string(), value: 10.0, active: true },
            DataRecord { id: 2, category: "B".to_string(), value: 20.0, active: true },
            DataRecord { id: 3, category: "A".to_string(), value: 30.0, active: false },
        ];

        let filtered = processor.filter_by_category("A");
        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().all(|r| r.category == "A"));
    }

    #[test]
    fn test_calculate_average() {
        let mut processor = DataProcessor::new();
        processor.records = vec![
            DataRecord { id: 1, category: "A".to_string(), value: 10.0, active: true },
            DataRecord { id: 2, category: "B".to_string(), value: 20.0, active: true },
            DataRecord { id: 3, category: "C".to_string(), value: 30.0, active: true },
        ];

        assert_eq!(processor.calculate_average(), Some(20.0));
    }

    #[test]
    fn test_empty_average() {
        let processor = DataProcessor::new();
        assert_eq!(processor.calculate_average(), None);
    }
}