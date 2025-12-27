
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, PartialEq)]
pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub category: String,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: &str) -> Result<Self, String> {
        if value < 0.0 {
            return Err(format!("Invalid value: {}", value));
        }
        if category.trim().is_empty() {
            return Err("Category cannot be empty".to_string());
        }
        Ok(Self {
            id,
            value,
            category: category.to_string(),
        })
    }
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        Self { records: Vec::new() }
    }

    pub fn load_from_csv(&mut self, file_path: &Path) -> Result<usize, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut count = 0;

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            if line_num == 0 {
                continue;
            }

            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() != 3 {
                continue;
            }

            let id = parts[0].parse::<u32>()?;
            let value = parts[1].parse::<f64>()?;
            let category = parts[2];

            match DataRecord::new(id, value, category) {
                Ok(record) => {
                    self.records.push(record);
                    count += 1;
                }
                Err(e) => eprintln!("Skipping invalid record at line {}: {}", line_num + 1, e),
            }
        }

        Ok(count)
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }

        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|r| r.category == category)
            .collect()
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
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_record_creation() {
        let record = DataRecord::new(1, 42.5, "test").unwrap();
        assert_eq!(record.id, 1);
        assert_eq!(record.value, 42.5);
        assert_eq!(record.category, "test");
    }

    #[test]
    fn test_invalid_data_record() {
        assert!(DataRecord::new(1, -5.0, "test").is_err());
        assert!(DataRecord::new(1, 10.0, "").is_err());
    }

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        assert_eq!(processor.get_record_count(), 0);

        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category").unwrap();
        writeln!(temp_file, "1,10.5,type_a").unwrap();
        writeln!(temp_file, "2,20.0,type_b").unwrap();
        writeln!(temp_file, "3,15.5,type_a").unwrap();

        let count = processor.load_from_csv(temp_file.path()).unwrap();
        assert_eq!(count, 3);
        assert_eq!(processor.get_record_count(), 3);

        let average = processor.calculate_average().unwrap();
        assert!((average - 15.3333).abs() < 0.001);

        let filtered = processor.filter_by_category("type_a");
        assert_eq!(filtered.len(), 2);

        processor.clear();
        assert_eq!(processor.get_record_count(), 0);
    }
}
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DataRecord {
    pub id: u64,
    pub timestamp: i64,
    pub values: HashMap<String, f64>,
    pub tags: Vec<String>,
}

#[derive(Debug, PartialEq)]
pub enum ValidationError {
    InvalidId,
    InvalidTimestamp,
    EmptyValues,
    DuplicateTags,
}

pub struct DataProcessor {
    validation_enabled: bool,
    max_value_count: usize,
}

impl DataProcessor {
    pub fn new(validation_enabled: bool, max_value_count: usize) -> Self {
        DataProcessor {
            validation_enabled,
            max_value_count,
        }
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), ValidationError> {
        if !self.validation_enabled {
            return Ok(());
        }

        if record.id == 0 {
            return Err(ValidationError::InvalidId);
        }

        if record.timestamp <= 0 {
            return Err(ValidationError::InvalidTimestamp);
        }

        if record.values.is_empty() {
            return Err(ValidationError::EmptyValues);
        }

        if record.values.len() > self.max_value_count {
            return Err(ValidationError::EmptyValues);
        }

        let unique_tags: std::collections::HashSet<_> = record.tags.iter().collect();
        if unique_tags.len() != record.tags.len() {
            return Err(ValidationError::DuplicateTags);
        }

        Ok(())
    }

    pub fn transform_values(&self, record: &DataRecord) -> HashMap<String, f64> {
        let mut transformed = HashMap::new();
        
        for (key, value) in &record.values {
            let transformed_value = match key.as_str() {
                "temperature" => (value - 32.0) * 5.0 / 9.0,
                "pressure" => value * 1000.0,
                "humidity" => value.min(100.0).max(0.0),
                _ => *value,
            };
            transformed.insert(key.clone(), transformed_value);
        }
        
        transformed
    }

    pub fn process_record(&self, record: DataRecord) -> Result<DataRecord, Box<dyn Error>> {
        self.validate_record(&record)?;
        
        let mut processed_record = record.clone();
        processed_record.values = self.transform_values(&record);
        
        Ok(processed_record)
    }

    pub fn batch_process(&self, records: Vec<DataRecord>) -> Vec<Result<DataRecord, Box<dyn Error>>> {
        records
            .into_iter()
            .map(|record| self.process_record(record))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_record() -> DataRecord {
        let mut values = HashMap::new();
        values.insert("temperature".to_string(), 68.0);
        values.insert("pressure".to_string(), 1.0);
        
        DataRecord {
            id: 1,
            timestamp: 1234567890,
            values,
            tags: vec!["sensor".to_string(), "room1".to_string()],
        }
    }

    #[test]
    fn test_validation_success() {
        let processor = DataProcessor::new(true, 10);
        let record = create_test_record();
        assert!(processor.validate_record(&record).is_ok());
    }

    #[test]
    fn test_validation_invalid_id() {
        let processor = DataProcessor::new(true, 10);
        let mut record = create_test_record();
        record.id = 0;
        assert_eq!(processor.validate_record(&record), Err(ValidationError::InvalidId));
    }

    #[test]
    fn test_transform_values() {
        let processor = DataProcessor::new(false, 10);
        let record = create_test_record();
        let transformed = processor.transform_values(&record);
        
        assert!((transformed.get("temperature").unwrap() - 20.0).abs() < 0.001);
        assert!((transformed.get("pressure").unwrap() - 1000.0).abs() < 0.001);
    }

    #[test]
    fn test_process_record() {
        let processor = DataProcessor::new(true, 10);
        let record = create_test_record();
        let result = processor.process_record(record);
        assert!(result.is_ok());
    }
}