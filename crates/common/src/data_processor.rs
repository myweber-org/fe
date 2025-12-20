
use std::error::Error;
use std::fs::File;
use std::path::Path;

pub struct DataRecord {
    id: u32,
    value: f64,
    timestamp: String,
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
        let path = Path::new(file_path);
        let file = File::open(path)?;
        let mut rdr = csv::Reader::from_reader(file);

        for result in rdr.deserialize() {
            let record: DataRecord = result?;
            self.records.push(record);
        }

        Ok(())
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }

        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn filter_by_threshold(&self, threshold: f64) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.value >= threshold)
            .collect()
    }

    pub fn record_count(&self) -> usize {
        self.records.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        assert_eq!(processor.record_count(), 0);
        assert_eq!(processor.calculate_average(), None);
    }

    #[test]
    fn test_csv_loading() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,timestamp").unwrap();
        writeln!(temp_file, "1,42.5,2024-01-15T10:30:00Z").unwrap();
        writeln!(temp_file, "2,37.8,2024-01-15T11:45:00Z").unwrap();

        let mut processor = DataProcessor::new();
        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        
        assert!(result.is_ok());
        assert_eq!(processor.record_count(), 2);
        
        let avg = processor.calculate_average();
        assert!(avg.is_some());
        assert!((avg.unwrap() - 40.15).abs() < 0.01);
    }

    #[test]
    fn test_threshold_filter() {
        let mut processor = DataProcessor::new();
        processor.records = vec![
            DataRecord { id: 1, value: 10.0, timestamp: String::from("2024-01-15") },
            DataRecord { id: 2, value: 20.0, timestamp: String::from("2024-01-15") },
            DataRecord { id: 3, value: 30.0, timestamp: String::from("2024-01-15") },
        ];

        let filtered = processor.filter_by_threshold(15.0);
        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().all(|r| r.value >= 15.0));
    }
}