use csv::Reader;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
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
        let mut rdr = Reader::from_path(path)?;
        for result in rdr.deserialize() {
            let record: Record = result?;
            self.records.push(record);
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

    pub fn validate_records(&self) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|record| record.value.is_finite() && !record.name.is_empty())
            .collect()
    }

    pub fn export_to_json<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn Error>> {
        let json = serde_json::to_string_pretty(&self.records)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    pub fn get_statistics(&self) -> Statistics {
        let count = self.records.len();
        let valid_count = self.validate_records().len();
        let avg_value = self.calculate_average().unwrap_or(0.0);
        let categories: std::collections::HashSet<_> = 
            self.records.iter().map(|r| r.category.clone()).collect();

        Statistics {
            total_records: count,
            valid_records: valid_count,
            average_value: avg_value,
            unique_categories: categories.len(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Statistics {
    pub total_records: usize,
    pub valid_records: usize,
    pub average_value: f64,
    pub unique_categories: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        let temp_file = NamedTempFile::new().unwrap();
        let test_data = "id,name,value,category\n1,Test1,10.5,A\n2,Test2,20.0,B\n";
        std::fs::write(temp_file.path(), test_data).unwrap();

        assert!(processor.load_from_csv(temp_file.path()).is_ok());
        assert_eq!(processor.records.len(), 2);
        
        let filtered = processor.filter_by_category("A");
        assert_eq!(filtered.len(), 1);
        
        let avg = processor.calculate_average();
        assert!(avg.is_some());
        assert!((avg.unwrap() - 15.25).abs() < 0.001);
    }
}
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRecord {
    pub id: u64,
    pub timestamp: i64,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

#[derive(Error, Debug)]
pub enum ProcessingError {
    #[error("Invalid data format")]
    InvalidFormat,
    #[error("Data validation failed: {0}")]
    ValidationFailed(String),
    #[error("Transformation error: {0}")]
    TransformationError(String),
}

pub struct DataProcessor {
    validation_threshold: f64,
    normalization_factor: f64,
}

impl DataProcessor {
    pub fn new(validation_threshold: f64, normalization_factor: f64) -> Self {
        Self {
            validation_threshold,
            normalization_factor,
        }
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if record.values.is_empty() {
            return Err(ProcessingError::ValidationFailed(
                "Empty values array".to_string(),
            ));
        }

        for value in &record.values {
            if value.is_nan() || value.is_infinite() {
                return Err(ProcessingError::ValidationFailed(
                    "Invalid numeric value".to_string(),
                ));
            }

            if value.abs() > self.validation_threshold {
                return Err(ProcessingError::ValidationFailed(format!(
                    "Value {} exceeds threshold {}",
                    value, self.validation_threshold
                )));
            }
        }

        Ok(())
    }

    pub fn normalize_values(&self, record: &mut DataRecord) {
        for value in &mut record.values {
            *value /= self.normalization_factor;
        }
    }

    pub fn calculate_statistics(&self, record: &DataRecord) -> HashMap<String, f64> {
        let mut stats = HashMap::new();

        if record.values.is_empty() {
            return stats;
        }

        let count = record.values.len() as f64;
        let sum: f64 = record.values.iter().sum();
        let mean = sum / count;

        let variance: f64 = record
            .values
            .iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>()
            / count;

        let min = record
            .values
            .iter()
            .fold(f64::INFINITY, |a, &b| a.min(b));
        let max = record
            .values
            .iter()
            .fold(f64::NEG_INFINITY, |a, &b| a.max(b));

        stats.insert("count".to_string(), count);
        stats.insert("sum".to_string(), sum);
        stats.insert("mean".to_string(), mean);
        stats.insert("variance".to_string(), variance);
        stats.insert("min".to_string(), min);
        stats.insert("max".to_string(), max);

        stats
    }

    pub fn process_record(&self, mut record: DataRecord) -> Result<DataRecord, ProcessingError> {
        self.validate_record(&record)?;
        self.normalize_values(&mut record);

        let stats = self.calculate_statistics(&record);
        record
            .metadata
            .insert("processed_stats".to_string(), format!("{:?}", stats));

        Ok(record)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_success() {
        let processor = DataProcessor::new(1000.0, 1.0);
        let record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: vec![10.0, 20.0, 30.0],
            metadata: HashMap::new(),
        };

        assert!(processor.validate_record(&record).is_ok());
    }

    #[test]
    fn test_validation_threshold_exceeded() {
        let processor = DataProcessor::new(10.0, 1.0);
        let record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: vec![5.0, 15.0, 25.0],
            metadata: HashMap::new(),
        };

        assert!(processor.validate_record(&record).is_err());
    }

    #[test]
    fn test_normalization() {
        let processor = DataProcessor::new(1000.0, 10.0);
        let mut record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: vec![10.0, 20.0, 30.0],
            metadata: HashMap::new(),
        };

        processor.normalize_values(&mut record);
        assert_eq!(record.values, vec![1.0, 2.0, 3.0]);
    }

    #[test]
    fn test_statistics_calculation() {
        let processor = DataProcessor::new(1000.0, 1.0);
        let record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: vec![1.0, 2.0, 3.0, 4.0, 5.0],
            metadata: HashMap::new(),
        };

        let stats = processor.calculate_statistics(&record);
        assert_eq!(stats.get("mean"), Some(&3.0));
        assert_eq!(stats.get("min"), Some(&1.0));
        assert_eq!(stats.get("max"), Some(&5.0));
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub category: String,
    pub timestamp: u64,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: String, timestamp: u64) -> Self {
        DataRecord {
            id,
            value,
            category,
            timestamp,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.id > 0 && self.value.is_finite() && !self.category.is_empty()
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

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<usize, Box<dyn Error>> {
        let path = Path::new(file_path);
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        
        let mut count = 0;
        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            
            if line_num == 0 {
                continue;
            }
            
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() != 4 {
                continue;
            }
            
            let id = match parts[0].parse::<u32>() {
                Ok(val) => val,
                Err(_) => continue,
            };
            
            let value = match parts[1].parse::<f64>() {
                Ok(val) => val,
                Err(_) => continue,
            };
            
            let category = parts[2].to_string();
            let timestamp = match parts[3].parse::<u64>() {
                Ok(val) => val,
                Err(_) => continue,
            };
            
            let record = DataRecord::new(id, value, category, timestamp);
            if record.is_valid() {
                self.records.push(record);
                count += 1;
            }
        }
        
        Ok(count)
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
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

    pub fn find_max_value(&self) -> Option<&DataRecord> {
        self.records.iter().max_by(|a, b| {
            a.value.partial_cmp(&b.value).unwrap()
        })
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
    fn test_data_record_validation() {
        let valid_record = DataRecord::new(1, 42.5, "test".to_string(), 1234567890);
        assert!(valid_record.is_valid());
        
        let invalid_record = DataRecord::new(0, 42.5, "test".to_string(), 1234567890);
        assert!(!invalid_record.is_valid());
    }

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        assert_eq!(processor.get_record_count(), 0);
        
        let csv_data = "id,value,category,timestamp\n1,10.5,type_a,1000\n2,20.3,type_b,2000\n3,15.7,type_a,3000\n";
        
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", csv_data).unwrap();
        
        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 3);
        assert_eq!(processor.get_record_count(), 3);
        
        let type_a_records = processor.filter_by_category("type_a");
        assert_eq!(type_a_records.len(), 2);
        
        let average = processor.calculate_average();
        assert!(average.is_some());
        assert!((average.unwrap() - 15.5).abs() < 0.1);
        
        let max_record = processor.find_max_value();
        assert!(max_record.is_some());
        assert_eq!(max_record.unwrap().value, 20.3);
        
        processor.clear();
        assert_eq!(processor.get_record_count(), 0);
    }
}