
use csv::Reader;
use serde::Deserialize;
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

pub fn process_data_file(file_path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let mut rdr = Reader::from_reader(file);
    let mut records = Vec::new();

    for result in rdr.deserialize() {
        let record: Record = result?;
        if record.value >= 0.0 {
            records.push(record);
        }
    }

    Ok(records)
}

pub fn calculate_statistics(records: &[Record]) -> (f64, f64, f64) {
    let count = records.len() as f64;
    if count == 0.0 {
        return (0.0, 0.0, 0.0);
    }

    let sum: f64 = records.iter().map(|r| r.value).sum();
    let mean = sum / count;
    
    let variance: f64 = records.iter()
        .map(|r| (r.value - mean).powi(2))
        .sum::<f64>() / count;
    
    let std_dev = variance.sqrt();
    
    (sum, mean, std_dev)
}

pub fn filter_by_category(records: Vec<Record>, category: &str) -> Vec<Record> {
    records.into_iter()
        .filter(|r| r.category == category)
        .collect()
}
use csv::Reader;
use serde::Deserialize;
use std::error::Error;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct Record {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub category: String,
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
        let mut reader = Reader::from_path(path)?;
        for result in reader.deserialize() {
            let record: Record = result?;
            self.validate_record(&record)?;
            self.records.push(record);
        }
        Ok(())
    }

    fn validate_record(&self, record: &Record) -> Result<(), String> {
        if record.name.is_empty() {
            return Err("Name cannot be empty".to_string());
        }
        if record.value < 0.0 {
            return Err("Value cannot be negative".to_string());
        }
        if record.category.is_empty() {
            return Err("Category cannot be empty".to_string());
        }
        Ok(())
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }
        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn get_statistics(&self) -> Statistics {
        let count = self.records.len();
        let avg = self.calculate_average().unwrap_or(0.0);
        let min = self.records.iter().map(|r| r.value).fold(f64::INFINITY, f64::min);
        let max = self.records.iter().map(|r| r.value).fold(f64::NEG_INFINITY, f64::max);

        Statistics {
            count,
            average: avg,
            min_value: min,
            max_value: max,
        }
    }

    pub fn records_count(&self) -> usize {
        self.records.len()
    }
}

pub struct Statistics {
    pub count: usize,
    pub average: f64,
    pub min_value: f64,
    pub max_value: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        assert_eq!(processor.records_count(), 0);

        let csv_data = "id,name,value,category\n1,ItemA,10.5,Category1\n2,ItemB,20.3,Category2\n";
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", csv_data).unwrap();

        let result = processor.load_from_csv(temp_file.path());
        assert!(result.is_ok());
        assert_eq!(processor.records_count(), 2);

        let stats = processor.get_statistics();
        assert_eq!(stats.count, 2);
        assert!((stats.average - 15.4).abs() < 0.001);
    }

    #[test]
    fn test_filter_by_category() {
        let mut processor = DataProcessor::new();
        let csv_data = "id,name,value,category\n1,ItemA,10.5,Category1\n2,ItemB,20.3,Category2\n3,ItemC,15.0,Category1\n";
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", csv_data).unwrap();

        processor.load_from_csv(temp_file.path()).unwrap();
        let filtered = processor.filter_by_category("Category1");
        assert_eq!(filtered.len(), 2);
    }
}