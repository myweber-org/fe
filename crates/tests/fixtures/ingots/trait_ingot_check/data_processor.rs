
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub timestamp: i64,
}

#[derive(Debug)]
pub enum ValidationError {
    InvalidId,
    InvalidValue,
    InvalidTimestamp,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::InvalidId => write!(f, "ID must be greater than 0"),
            ValidationError::InvalidValue => write!(f, "Value must be between 0.0 and 1000.0"),
            ValidationError::InvalidTimestamp => write!(f, "Timestamp must be positive"),
        }
    }
}

impl Error for ValidationError {}

pub fn validate_record(record: &DataRecord) -> Result<(), ValidationError> {
    if record.id == 0 {
        return Err(ValidationError::InvalidId);
    }
    
    if record.value < 0.0 || record.value > 1000.0 {
        return Err(ValidationError::InvalidValue);
    }
    
    if record.timestamp <= 0 {
        return Err(ValidationError::InvalidTimestamp);
    }
    
    Ok(())
}

pub fn transform_record(record: &DataRecord) -> DataRecord {
    DataRecord {
        id: record.id,
        value: (record.value * 100.0).round() / 100.0,
        timestamp: record.timestamp,
    }
}

pub fn process_records(records: Vec<DataRecord>) -> Vec<Result<DataRecord, ValidationError>> {
    records
        .into_iter()
        .map(|record| {
            validate_record(&record)?;
            Ok(transform_record(&record))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_record() {
        let record = DataRecord {
            id: 1,
            value: 500.0,
            timestamp: 1234567890,
        };
        
        assert!(validate_record(&record).is_ok());
    }

    #[test]
    fn test_invalid_id() {
        let record = DataRecord {
            id: 0,
            value: 500.0,
            timestamp: 1234567890,
        };
        
        assert!(matches!(
            validate_record(&record),
            Err(ValidationError::InvalidId)
        ));
    }

    #[test]
    fn test_transform_record() {
        let record = DataRecord {
            id: 1,
            value: 123.4567,
            timestamp: 1234567890,
        };
        
        let transformed = transform_record(&record);
        assert_eq!(transformed.value, 123.46);
    }
}
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

pub struct DataProcessor {
    records: Vec<DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<usize, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut count = 0;

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            
            if line_num == 0 {
                continue;
            }

            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() != 3 {
                return Err(format!("Invalid CSV format at line {}", line_num + 1).into());
            }

            let id = parts[0].parse::<u32>()?;
            let value = parts[1].parse::<f64>()?;
            let category = parts[2].to_string();

            if value < 0.0 {
                return Err(format!("Negative value at line {}", line_num + 1).into());
            }

            self.records.push(DataRecord { id, value, category });
            count += 1;
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
            .filter(|record| record.category == category)
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
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        assert_eq!(processor.get_record_count(), 0);

        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category").unwrap();
        writeln!(temp_file, "1,10.5,TypeA").unwrap();
        writeln!(temp_file, "2,20.0,TypeB").unwrap();
        writeln!(temp_file, "3,15.75,TypeA").unwrap();

        let result = processor.load_from_csv(temp_file.path());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 3);
        assert_eq!(processor.get_record_count(), 3);

        let average = processor.calculate_average();
        assert!(average.is_some());
        assert!((average.unwrap() - 15.416666666666666).abs() < 0.0001);

        let type_a_records = processor.filter_by_category("TypeA");
        assert_eq!(type_a_records.len(), 2);

        processor.clear();
        assert_eq!(processor.get_record_count(), 0);
    }

    #[test]
    fn test_invalid_csv() {
        let mut processor = DataProcessor::new();
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category").unwrap();
        writeln!(temp_file, "1,invalid,TypeA").unwrap();

        let result = processor.load_from_csv(temp_file.path());
        assert!(result.is_err());
    }
}