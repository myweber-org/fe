
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

#[derive(Debug, Error)]
pub enum ProcessingError {
    #[error("Invalid data format")]
    InvalidFormat,
    #[error("Data validation failed: {0}")]
    ValidationFailed(String),
    #[error("Transformation error: {0}")]
    TransformationError(String),
}

pub struct DataProcessor {
    config: ProcessingConfig,
}

#[derive(Debug, Clone)]
pub struct ProcessingConfig {
    pub max_values: usize,
    pub min_timestamp: i64,
    pub require_metadata: bool,
}

impl DataProcessor {
    pub fn new(config: ProcessingConfig) -> Self {
        DataProcessor { config }
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if record.values.len() > self.config.max_values {
            return Err(ProcessingError::ValidationFailed(
                format!("Too many values: {}", record.values.len())
            ));
        }

        if record.timestamp < self.config.min_timestamp {
            return Err(ProcessingError::ValidationFailed(
                format!("Timestamp too old: {}", record.timestamp)
            ));
        }

        if self.config.require_metadata && record.metadata.is_empty() {
            return Err(ProcessingError::ValidationFailed(
                "Metadata required but missing".to_string()
            ));
        }

        Ok(())
    }

    pub fn transform_record(&self, record: &DataRecord) -> Result<DataRecord, ProcessingError> {
        let mut transformed = record.clone();
        
        transformed.values = record.values
            .iter()
            .map(|&v| v * 2.0)
            .collect();

        transformed.metadata.insert(
            "processed_at".to_string(),
            chrono::Utc::now().to_rfc3339()
        );

        transformed.metadata.insert(
            "transformation_count".to_string(),
            "1".to_string()
        );

        Ok(transformed)
    }

    pub fn process_batch(&self, records: Vec<DataRecord>) -> Result<Vec<DataRecord>, ProcessingError> {
        let mut processed = Vec::with_capacity(records.len());

        for record in records {
            self.validate_record(&record)?;
            let transformed = self.transform_record(&record)?;
            processed.push(transformed);
        }

        Ok(processed)
    }

    pub fn calculate_statistics(&self, records: &[DataRecord]) -> HashMap<String, f64> {
        let mut stats = HashMap::new();

        let total_records = records.len() as f64;
        if total_records == 0.0 {
            return stats;
        }

        let total_values: usize = records.iter()
            .map(|r| r.values.len())
            .sum();

        let sum_all_values: f64 = records.iter()
            .flat_map(|r| &r.values)
            .sum();

        let avg_values_per_record = total_values as f64 / total_records;
        let avg_value = sum_all_values / total_values as f64;

        stats.insert("total_records".to_string(), total_records);
        stats.insert("total_values".to_string(), total_values as f64);
        stats.insert("avg_values_per_record".to_string(), avg_values_per_record);
        stats.insert("avg_value".to_string(), avg_value);

        stats
    }
}

impl Default for ProcessingConfig {
    fn default() -> Self {
        ProcessingConfig {
            max_values: 100,
            min_timestamp: 0,
            require_metadata: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_success() {
        let config = ProcessingConfig::default();
        let processor = DataProcessor::new(config);

        let record = DataRecord {
            id: 1,
            timestamp: 1000,
            values: vec![1.0, 2.0, 3.0],
            metadata: HashMap::new(),
        };

        assert!(processor.validate_record(&record).is_ok());
    }

    #[test]
    fn test_validation_too_many_values() {
        let config = ProcessingConfig {
            max_values: 2,
            ..ProcessingConfig::default()
        };
        let processor = DataProcessor::new(config);

        let record = DataRecord {
            id: 1,
            timestamp: 1000,
            values: vec![1.0, 2.0, 3.0],
            metadata: HashMap::new(),
        };

        assert!(processor.validate_record(&record).is_err());
    }

    #[test]
    fn test_transform_record() {
        let processor = DataProcessor::new(ProcessingConfig::default());
        
        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), "test".to_string());

        let record = DataRecord {
            id: 1,
            timestamp: 1000,
            values: vec![1.0, 2.0, 3.0],
            metadata,
        };

        let transformed = processor.transform_record(&record).unwrap();
        
        assert_eq!(transformed.values, vec![2.0, 4.0, 6.0]);
        assert!(transformed.metadata.contains_key("processed_at"));
        assert!(transformed.metadata.contains_key("transformation_count"));
    }

    #[test]
    fn test_calculate_statistics() {
        let processor = DataProcessor::new(ProcessingConfig::default());

        let records = vec![
            DataRecord {
                id: 1,
                timestamp: 1000,
                values: vec![1.0, 2.0],
                metadata: HashMap::new(),
            },
            DataRecord {
                id: 2,
                timestamp: 2000,
                values: vec![3.0, 4.0, 5.0],
                metadata: HashMap::new(),
            },
        ];

        let stats = processor.calculate_statistics(&records);

        assert_eq!(stats.get("total_records").unwrap(), &2.0);
        assert_eq!(stats.get("total_values").unwrap(), &5.0);
        assert_eq!(stats.get("avg_values_per_record").unwrap(), &2.5);
        assert_eq!(stats.get("avg_value").unwrap(), &3.0);
    }
}use std::error::Error;
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
        let mut lines = reader.lines();

        if self.has_header {
            lines.next();
        }

        for line_result in lines {
            let line = line_result?;
            let fields: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();
            
            if !fields.is_empty() && !fields.iter().all(|f| f.is_empty()) {
                records.push(fields);
            }
        }

        Ok(records)
    }

    pub fn validate_record(&self, record: &[String]) -> bool {
        !record.is_empty() && record.iter().all(|field| !field.is_empty())
    }

    pub fn extract_column(&self, data: &[Vec<String>], column_index: usize) -> Vec<String> {
        data.iter()
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
    fn test_process_csv() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,30,New York").unwrap();
        writeln!(temp_file, "Bob,25,London").unwrap();

        let processor = DataProcessor::new(',', true);
        let result = processor.process_file(temp_file.path()).unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["Alice", "30", "New York"]);
        assert_eq!(result[1], vec!["Bob", "25", "London"]);
    }

    #[test]
    fn test_validate_record() {
        let processor = DataProcessor::new(',', false);
        let valid_record = vec!["data".to_string(), "value".to_string()];
        let invalid_record = vec!["".to_string(), "value".to_string()];

        assert!(processor.validate_record(&valid_record));
        assert!(!processor.validate_record(&invalid_record));
    }

    #[test]
    fn test_extract_column() {
        let data = vec![
            vec!["a".to_string(), "b".to_string(), "c".to_string()],
            vec!["d".to_string(), "e".to_string(), "f".to_string()],
        ];
        
        let processor = DataProcessor::new(',', false);
        let column = processor.extract_column(&data, 1);
        
        assert_eq!(column, vec!["b".to_string(), "e".to_string()]);
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
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

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<usize, Box<dyn Error>> {
        let path = Path::new(file_path);
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        
        let mut count = 0;
        for (index, line) in reader.lines().enumerate() {
            let line = line?;
            
            if index == 0 {
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
            
            let name = parts[1].to_string();
            
            let value = match parts[2].parse::<f64>() {
                Ok(val) => val,
                Err(_) => continue,
            };
            
            let category = parts[3].to_string();
            
            self.records.push(Record {
                id,
                name,
                value,
                category,
            });
            
            count += 1;
        }
        
        Ok(count)
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<Record> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .cloned()
            .collect()
    }

    pub fn calculate_average(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }
        
        let sum: f64 = self.records.iter().map(|record| record.value).sum();
        sum / self.records.len() as f64
    }

    pub fn find_max_value(&self) -> Option<&Record> {
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
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        assert_eq!(processor.get_record_count(), 0);
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,category").unwrap();
        writeln!(temp_file, "1,ItemA,25.5,Electronics").unwrap();
        writeln!(temp_file, "2,ItemB,42.8,Electronics").unwrap();
        writeln!(temp_file, "3,ItemC,15.2,Books").unwrap();
        
        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(processor.get_record_count(), 3);
        
        let electronics = processor.filter_by_category("Electronics");
        assert_eq!(electronics.len(), 2);
        
        let average = processor.calculate_average();
        assert!(average > 0.0);
        
        let max_record = processor.find_max_value();
        assert!(max_record.is_some());
        assert_eq!(max_record.unwrap().id, 2);
    }
}