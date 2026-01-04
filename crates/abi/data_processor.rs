
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DataRecord {
    pub id: u64,
    pub timestamp: i64,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

impl DataRecord {
    pub fn new(id: u64, timestamp: i64) -> Self {
        Self {
            id,
            timestamp,
            values: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn add_value(&mut self, value: f64) -> &mut Self {
        self.values.push(value);
        self
    }

    pub fn add_metadata(&mut self, key: String, value: String) -> &mut Self {
        self.metadata.insert(key, value);
        self
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.id == 0 {
            return Err("ID cannot be zero".to_string());
        }
        if self.timestamp < 0 {
            return Err("Timestamp cannot be negative".to_string());
        }
        if self.values.is_empty() {
            return Err("Values cannot be empty".to_string());
        }
        Ok(())
    }

    pub fn calculate_statistics(&self) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        
        if self.values.is_empty() {
            return stats;
        }

        let sum: f64 = self.values.iter().sum();
        let count = self.values.len() as f64;
        let mean = sum / count;

        let variance: f64 = self.values
            .iter()
            .map(|&v| (v - mean).powi(2))
            .sum::<f64>() / count;

        let min = self.values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = self.values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

        stats.insert("mean".to_string(), mean);
        stats.insert("variance".to_string(), variance);
        stats.insert("min".to_string(), min);
        stats.insert("max".to_string(), max);
        stats.insert("count".to_string(), count);
        stats.insert("sum".to_string(), sum);

        stats
    }
}

pub fn process_records(records: &[DataRecord]) -> Vec<HashMap<String, f64>> {
    records
        .iter()
        .filter(|record| record.validate().is_ok())
        .map(|record| record.calculate_statistics())
        .collect()
}

pub fn filter_by_metadata(
    records: &[DataRecord],
    key: &str,
    value: &str,
) -> Vec<DataRecord> {
    records
        .iter()
        .filter(|record| {
            record.metadata
                .get(key)
                .map_or(false, |v| v == value)
        })
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_record_creation() {
        let record = DataRecord::new(1, 1234567890);
        assert_eq!(record.id, 1);
        assert_eq!(record.timestamp, 1234567890);
        assert!(record.values.is_empty());
        assert!(record.metadata.is_empty());
    }

    #[test]
    fn test_add_value_and_metadata() {
        let mut record = DataRecord::new(1, 1234567890);
        record.add_value(42.0)
              .add_value(24.0)
              .add_metadata("source".to_string(), "test".to_string());

        assert_eq!(record.values.len(), 2);
        assert_eq!(record.values[0], 42.0);
        assert_eq!(record.values[1], 24.0);
        assert_eq!(record.metadata.get("source"), Some(&"test".to_string()));
    }

    #[test]
    fn test_validation() {
        let valid_record = DataRecord::new(1, 1234567890);
        valid_record.add_value(10.0);
        assert!(valid_record.validate().is_ok());

        let invalid_record = DataRecord::new(0, 1234567890);
        invalid_record.add_value(10.0);
        assert!(invalid_record.validate().is_err());
    }

    #[test]
    fn test_calculate_statistics() {
        let mut record = DataRecord::new(1, 1234567890);
        record.add_value(10.0).add_value(20.0).add_value(30.0);

        let stats = record.calculate_statistics();
        assert_eq!(stats.get("mean"), Some(&20.0));
        assert_eq!(stats.get("min"), Some(&10.0));
        assert_eq!(stats.get("max"), Some(&30.0));
        assert_eq!(stats.get("count"), Some(&3.0));
    }

    #[test]
    fn test_filter_by_metadata() {
        let mut record1 = DataRecord::new(1, 1234567890);
        record1.add_metadata("type".to_string(), "a".to_string());

        let mut record2 = DataRecord::new(2, 1234567891);
        record2.add_metadata("type".to_string(), "b".to_string());

        let records = vec![record1, record2];
        let filtered = filter_by_metadata(&records, "type", "a");

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, 1);
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct DataProcessor {
    delimiter: char,
    has_header: bool,
}

impl DataProcessor {
    pub fn new(delimiter: char, has_header: bool) -> Self {
        DataProcessor {
            delimiter,
            has_header,
        }
    }

    pub fn process_file<P: AsRef<Path>>(&self, file_path: P) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();

        for (line_number, line_result) in reader.lines().enumerate() {
            let line = line_result?;
            
            if line_number == 0 && self.has_header {
                continue;
            }

            if line.trim().is_empty() {
                continue;
            }

            let fields: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            if fields.iter().all(|f| f.is_empty()) {
                continue;
            }

            records.push(fields);
        }

        Ok(records)
    }

    pub fn validate_record(&self, record: &[String]) -> bool {
        !record.is_empty() && record.iter().any(|field| !field.is_empty())
    }

    pub fn extract_column(&self, records: &[Vec<String>], column_index: usize) -> Vec<String> {
        records
            .iter()
            .filter_map(|record| record.get(column_index).cloned())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_process_file_with_header() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,30,New York").unwrap();
        writeln!(temp_file, "Bob,25,London").unwrap();

        let processor = DataProcessor::new(',', true);
        let result = processor.process_file(temp_file.path()).unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["Alice", "30", "New York"]);
    }

    #[test]
    fn test_validate_record() {
        let processor = DataProcessor::new(',', false);
        let valid_record = vec!["data".to_string(), "value".to_string()];
        let empty_record = vec![];
        let blank_record = vec!["".to_string(), "".to_string()];

        assert!(processor.validate_record(&valid_record));
        assert!(!processor.validate_record(&empty_record));
        assert!(!processor.validate_record(&blank_record));
    }

    #[test]
    fn test_extract_column() {
        let records = vec![
            vec!["a".to_string(), "b".to_string(), "c".to_string()],
            vec!["d".to_string(), "e".to_string(), "f".to_string()],
        ];
        
        let processor = DataProcessor::new(',', false);
        let column = processor.extract_column(&records, 1);
        
        assert_eq!(column, vec!["b".to_string(), "e".to_string()]);
    }
}