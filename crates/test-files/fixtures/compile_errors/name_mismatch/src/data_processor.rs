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
        let mut lines = reader.lines();

        if self.has_header {
            lines.next();
        }

        for line_result in lines {
            let line = line_result?;
            let fields: Vec<String> = line.split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();
            
            if !fields.is_empty() {
                records.push(fields);
            }
        }

        Ok(records)
    }

    pub fn validate_record(&self, record: &[String], expected_fields: usize) -> bool {
        record.len() == expected_fields && 
        record.iter().all(|field| !field.is_empty())
    }

    pub fn extract_column(&self, data: &[Vec<String>], column_index: usize) -> Vec<String> {
        data.iter()
            .filter_map(|record| record.get(column_index))
            .cloned()
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
        let valid_record = vec!["field1".to_string(), "field2".to_string()];
        let invalid_record = vec!["".to_string(), "field2".to_string()];
        
        assert!(processor.validate_record(&valid_record, 2));
        assert!(!processor.validate_record(&invalid_record, 2));
    }

    #[test]
    fn test_extract_column() {
        let processor = DataProcessor::new(',', false);
        let data = vec![
            vec!["a".to_string(), "b".to_string()],
            vec!["c".to_string(), "d".to_string()],
        ];
        
        let column = processor.extract_column(&data, 0);
        assert_eq!(column, vec!["a".to_string(), "c".to_string()]);
    }
}
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DataError {
    #[error("Invalid data format")]
    InvalidFormat,
    #[error("Data validation failed: {0}")]
    ValidationFailed(String),
    #[error("Transformation error: {0}")]
    TransformationError(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRecord {
    pub id: u64,
    pub value: f64,
    pub timestamp: i64,
    pub metadata: Option<String>,
}

impl DataRecord {
    pub fn validate(&self) -> Result<(), DataError> {
        if self.id == 0 {
            return Err(DataError::ValidationFailed("ID cannot be zero".to_string()));
        }
        
        if self.value.is_nan() || self.value.is_infinite() {
            return Err(DataError::ValidationFailed("Value must be finite".to_string()));
        }
        
        if self.timestamp < 0 {
            return Err(DataError::ValidationFailed("Timestamp cannot be negative".to_string()));
        }
        
        Ok(())
    }
    
    pub fn transform(&mut self, multiplier: f64) -> Result<(), DataError> {
        if multiplier <= 0.0 {
            return Err(DataError::TransformationError(
                "Multiplier must be positive".to_string()
            ));
        }
        
        self.value *= multiplier;
        self.timestamp += 1;
        
        Ok(())
    }
}

pub fn process_records(records: &mut [DataRecord], multiplier: f64) -> Result<Vec<DataRecord>, DataError> {
    let mut processed = Vec::with_capacity(records.len());
    
    for record in records.iter_mut() {
        record.validate()?;
        record.transform(multiplier)?;
        processed.push(record.clone());
    }
    
    Ok(processed)
}

pub fn filter_records(records: &[DataRecord], min_value: f64) -> Vec<DataRecord> {
    records
        .iter()
        .filter(|r| r.value >= min_value)
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_valid_record() {
        let record = DataRecord {
            id: 1,
            value: 42.0,
            timestamp: 1234567890,
            metadata: Some("test".to_string()),
        };
        
        assert!(record.validate().is_ok());
    }
    
    #[test]
    fn test_invalid_record() {
        let record = DataRecord {
            id: 0,
            value: f64::NAN,
            timestamp: -1,
            metadata: None,
        };
        
        assert!(record.validate().is_err());
    }
    
    #[test]
    fn test_transform_record() {
        let mut record = DataRecord {
            id: 1,
            value: 10.0,
            timestamp: 1000,
            metadata: None,
        };
        
        assert!(record.transform(2.0).is_ok());
        assert_eq!(record.value, 20.0);
        assert_eq!(record.timestamp, 1001);
    }
    
    #[test]
    fn test_filter_records() {
        let records = vec![
            DataRecord { id: 1, value: 5.0, timestamp: 1000, metadata: None },
            DataRecord { id: 2, value: 15.0, timestamp: 1001, metadata: None },
            DataRecord { id: 3, value: 25.0, timestamp: 1002, metadata: None },
        ];
        
        let filtered = filter_records(&records, 10.0);
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0].id, 2);
        assert_eq!(filtered[1].id, 3);
    }
}