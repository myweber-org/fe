
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

    pub fn validate_records(&self, records: &[Vec<String>], expected_columns: usize) -> Result<(), String> {
        for (index, record) in records.iter().enumerate() {
            if record.len() != expected_columns {
                return Err(format!(
                    "Record {} has {} columns, expected {}",
                    index + 1,
                    record.len(),
                    expected_columns
                ));
            }
        }
        Ok(())
    }

    pub fn extract_column(&self, records: &[Vec<String>], column_index: usize) -> Result<Vec<String>, String> {
        if records.is_empty() {
            return Ok(Vec::new());
        }

        let mut column_data = Vec::with_capacity(records.len());
        
        for (row_index, record) in records.iter().enumerate() {
            if column_index >= record.len() {
                return Err(format!(
                    "Column index {} out of bounds for record {} (max index: {})",
                    column_index,
                    row_index + 1,
                    record.len() - 1
                ));
            }
            column_data.push(record[column_index].clone());
        }
        
        Ok(column_data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_process_csv_with_header() {
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
    fn test_validate_records() {
        let records = vec![
            vec!["a".to_string(), "b".to_string(), "c".to_string()],
            vec!["d".to_string(), "e".to_string(), "f".to_string()],
        ];
        
        let processor = DataProcessor::new(',', false);
        assert!(processor.validate_records(&records, 3).is_ok());
        assert!(processor.validate_records(&records, 2).is_err());
    }

    #[test]
    fn test_extract_column() {
        let records = vec![
            vec!["a".to_string(), "b".to_string(), "c".to_string()],
            vec!["d".to_string(), "e".to_string(), "f".to_string()],
        ];
        
        let processor = DataProcessor::new(',', false);
        let column = processor.extract_column(&records, 1).unwrap();
        
        assert_eq!(column, vec!["b", "e"]);
    }
}
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRecord {
    pub id: u64,
    pub value: f64,
    pub timestamp: i64,
}

#[derive(Debug)]
pub enum ProcessingError {
    InvalidValue,
    InvalidTimestamp,
    ValidationFailed(String),
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessingError::InvalidValue => write!(f, "Invalid numeric value"),
            ProcessingError::InvalidTimestamp => write!(f, "Invalid timestamp"),
            ProcessingError::ValidationFailed(msg) => write!(f, "Validation failed: {}", msg),
        }
    }
}

impl Error for ProcessingError {}

pub struct DataProcessor {
    threshold: f64,
}

impl DataProcessor {
    pub fn new(threshold: f64) -> Self {
        DataProcessor { threshold }
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if record.value.is_nan() || record.value.is_infinite() {
            return Err(ProcessingError::InvalidValue);
        }

        if record.timestamp < 0 {
            return Err(ProcessingError::InvalidTimestamp);
        }

        if record.value.abs() > self.threshold {
            return Err(ProcessingError::ValidationFailed(
                format!("Value {} exceeds threshold {}", record.value, self.threshold)
            ));
        }

        Ok(())
    }

    pub fn transform_records(&self, records: Vec<DataRecord>) -> Vec<DataRecord> {
        records
            .into_iter()
            .filter_map(|mut record| {
                if self.validate_record(&record).is_ok() {
                    record.value = (record.value * 100.0).round() / 100.0;
                    Some(record)
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn calculate_statistics(&self, records: &[DataRecord]) -> Option<(f64, f64, f64)> {
        if records.is_empty() {
            return None;
        }

        let count = records.len() as f64;
        let sum: f64 = records.iter().map(|r| r.value).sum();
        let mean = sum / count;

        let variance: f64 = records.iter()
            .map(|r| (r.value - mean).powi(2))
            .sum::<f64>() / count;

        let std_dev = variance.sqrt();

        Some((mean, variance, std_dev))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_success() {
        let processor = DataProcessor::new(1000.0);
        let record = DataRecord {
            id: 1,
            value: 500.0,
            timestamp: 1625097600,
        };
        
        assert!(processor.validate_record(&record).is_ok());
    }

    #[test]
    fn test_validation_threshold_exceeded() {
        let processor = DataProcessor::new(100.0);
        let record = DataRecord {
            id: 1,
            value: 150.0,
            timestamp: 1625097600,
        };
        
        match processor.validate_record(&record) {
            Err(ProcessingError::ValidationFailed(_)) => (),
            _ => panic!("Expected ValidationFailed error"),
        }
    }

    #[test]
    fn test_transform_records() {
        let processor = DataProcessor::new(1000.0);
        let records = vec![
            DataRecord { id: 1, value: 123.456, timestamp: 1625097600 },
            DataRecord { id: 2, value: 789.012, timestamp: 1625097601 },
        ];
        
        let transformed = processor.transform_records(records);
        assert_eq!(transformed.len(), 2);
        assert_eq!(transformed[0].value, 123.46);
        assert_eq!(transformed[1].value, 789.01);
    }

    #[test]
    fn test_statistics_calculation() {
        let processor = DataProcessor::new(1000.0);
        let records = vec![
            DataRecord { id: 1, value: 10.0, timestamp: 1625097600 },
            DataRecord { id: 2, value: 20.0, timestamp: 1625097601 },
            DataRecord { id: 3, value: 30.0, timestamp: 1625097602 },
        ];
        
        let stats = processor.calculate_statistics(&records).unwrap();
        assert_eq!(stats.0, 20.0);
        assert_eq!(stats.1, 66.66666666666667);
        assert_eq!(stats.2, 8.16496580927726);
    }
}