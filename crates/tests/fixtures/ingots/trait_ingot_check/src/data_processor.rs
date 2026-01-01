
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DataRecord {
    pub id: u64,
    pub timestamp: i64,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug)]
pub struct ProcessedData {
    pub record_id: u64,
    pub normalized_values: Vec<f64>,
    pub is_valid: bool,
    pub processing_time_ms: u64,
}

pub struct DataProcessor {
    validation_threshold: f64,
    normalization_factor: f64,
}

impl DataProcessor {
    pub fn new(validation_threshold: f64, normalization_factor: f64) -> Self {
        DataProcessor {
            validation_threshold,
            normalization_factor,
        }
    }

    pub fn process_record(&self, record: &DataRecord) -> Result<ProcessedData, Box<dyn Error>> {
        let start_time = std::time::Instant::now();
        
        let is_valid = self.validate_record(record);
        let normalized_values = if is_valid {
            self.normalize_values(&record.values)
        } else {
            Vec::new()
        };
        
        let processing_time = start_time.elapsed().as_millis() as u64;
        
        Ok(ProcessedData {
            record_id: record.id,
            normalized_values,
            is_valid,
            processing_time_ms: processing_time,
        })
    }

    fn validate_record(&self, record: &DataRecord) -> bool {
        if record.values.is_empty() {
            return false;
        }
        
        for value in &record.values {
            if value.is_nan() || value.is_infinite() {
                return false;
            }
            
            if value.abs() > self.validation_threshold {
                return false;
            }
        }
        
        true
    }

    fn normalize_values(&self, values: &[f64]) -> Vec<f64> {
        values
            .iter()
            .map(|&v| v * self.normalization_factor)
            .collect()
    }

    pub fn batch_process(&self, records: &[DataRecord]) -> Vec<Result<ProcessedData, Box<dyn Error>>> {
        records
            .iter()
            .map(|record| self.process_record(record))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_processor_validation() {
        let processor = DataProcessor::new(1000.0, 2.0);
        
        let valid_record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: vec![10.5, 20.3, 30.7],
            metadata: HashMap::new(),
        };
        
        let invalid_record = DataRecord {
            id: 2,
            timestamp: 1234567891,
            values: vec![f64::NAN, 20.3],
            metadata: HashMap::new(),
        };
        
        assert!(processor.validate_record(&valid_record));
        assert!(!processor.validate_record(&invalid_record));
    }

    #[test]
    fn test_normalization() {
        let processor = DataProcessor::new(1000.0, 0.5);
        let values = vec![2.0, 4.0, 6.0];
        let normalized = processor.normalize_values(&values);
        
        assert_eq!(normalized, vec![1.0, 2.0, 3.0]);
    }

    #[test]
    fn test_process_record() {
        let processor = DataProcessor::new(1000.0, 2.0);
        
        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), "test".to_string());
        
        let record = DataRecord {
            id: 42,
            timestamp: 1234567890,
            values: vec![1.0, 2.0, 3.0],
            metadata,
        };
        
        let result = processor.process_record(&record).unwrap();
        
        assert_eq!(result.record_id, 42);
        assert_eq!(result.normalized_values, vec![2.0, 4.0, 6.0]);
        assert!(result.is_valid);
        assert!(result.processing_time_ms > 0);
    }
}use std::error::Error;
use std::fs::File;
use std::path::Path;

pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub category: String,
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

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|r| r.category == category)
            .collect()
    }

    pub fn validate_records(&self) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|r| r.value >= 0.0 && r.value <= 100.0)
            .collect()
    }

    pub fn count_records(&self) -> usize {
        self.records.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processing() {
        let mut processor = DataProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category").unwrap();
        writeln!(temp_file, "1,85.5,CategoryA").unwrap();
        writeln!(temp_file, "2,92.3,CategoryB").unwrap();
        writeln!(temp_file, "3,78.9,CategoryA").unwrap();
        
        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(processor.count_records(), 3);
        
        let avg = processor.calculate_average();
        assert!(avg.is_some());
        assert!((avg.unwrap() - 85.5667).abs() < 0.001);
        
        let category_a = processor.filter_by_category("CategoryA");
        assert_eq!(category_a.len(), 2);
        
        let valid = processor.validate_records();
        assert_eq!(valid.len(), 3);
    }
}