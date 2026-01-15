use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct DataProcessor {
    file_path: String,
}

impl DataProcessor {
    pub fn new(file_path: &str) -> Self {
        DataProcessor {
            file_path: file_path.to_string(),
        }
    }

    pub fn process_csv(&self, filter_column: usize, filter_value: &str) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let path = Path::new(&self.file_path);
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        
        let mut filtered_data = Vec::new();
        
        for line in reader.lines() {
            let line = line?;
            let columns: Vec<String> = line.split(',').map(|s| s.trim().to_string()).collect();
            
            if columns.len() > filter_column && columns[filter_column] == filter_value {
                filtered_data.push(columns);
            }
        }
        
        Ok(filtered_data)
    }

    pub fn calculate_average(&self, column_index: usize) -> Result<f64, Box<dyn Error>> {
        let path = Path::new(&self.file_path);
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        
        let mut sum = 0.0;
        let mut count = 0;
        
        for line in reader.lines().skip(1) {
            let line = line?;
            let columns: Vec<&str> = line.split(',').collect();
            
            if let Some(value_str) = columns.get(column_index) {
                if let Ok(value) = value_str.trim().parse::<f64>() {
                    sum += value;
                    count += 1;
                }
            }
        }
        
        if count > 0 {
            Ok(sum / count as f64)
        } else {
            Ok(0.0)
        }
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
        writeln!(temp_file, "id,name,age,department").unwrap();
        writeln!(temp_file, "1,John,30,Engineering").unwrap();
        writeln!(temp_file, "2,Jane,25,Marketing").unwrap();
        writeln!(temp_file, "3,Bob,35,Engineering").unwrap();
        
        let processor = DataProcessor::new(temp_file.path().to_str().unwrap());
        let result = processor.process_csv(3, "Engineering").unwrap();
        
        assert_eq!(result.len(), 2);
        assert_eq!(result[0][1], "John");
        assert_eq!(result[1][1], "Bob");
    }

    #[test]
    fn test_calculate_average() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,age").unwrap();
        writeln!(temp_file, "1,John,30").unwrap();
        writeln!(temp_file, "2,Jane,25").unwrap();
        writeln!(temp_file, "3,Bob,35").unwrap();
        
        let processor = DataProcessor::new(temp_file.path().to_str().unwrap());
        let average = processor.calculate_average(2).unwrap();
        
        assert!((average - 30.0).abs() < 0.001);
    }
}
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub category: String,
}

#[derive(Debug)]
pub enum ValidationError {
    InvalidId,
    InvalidValue,
    EmptyCategory,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::InvalidId => write!(f, "ID must be greater than 0"),
            ValidationError::InvalidValue => write!(f, "Value must be between 0.0 and 1000.0"),
            ValidationError::EmptyCategory => write!(f, "Category cannot be empty"),
        }
    }
}

impl Error for ValidationError {}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: String) -> Result<Self, ValidationError> {
        if id == 0 {
            return Err(ValidationError::InvalidId);
        }
        
        if value < 0.0 || value > 1000.0 {
            return Err(ValidationError::InvalidValue);
        }
        
        if category.trim().is_empty() {
            return Err(ValidationError::EmptyCategory);
        }
        
        Ok(Self {
            id,
            value,
            category: category.trim().to_string(),
        })
    }
    
    pub fn normalize_value(&self) -> f64 {
        self.value / 1000.0
    }
    
    pub fn to_uppercase_category(&self) -> String {
        self.category.to_uppercase()
    }
}

pub fn process_records(records: &[DataRecord]) -> Vec<(u32, f64, String)> {
    records
        .iter()
        .map(|record| {
            (
                record.id,
                record.normalize_value(),
                record.to_uppercase_category(),
            )
        })
        .collect()
}

pub fn filter_by_threshold(records: &[DataRecord], threshold: f64) -> Vec<&DataRecord> {
    records
        .iter()
        .filter(|record| record.value >= threshold)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_valid_record_creation() {
        let record = DataRecord::new(1, 500.0, "Test".to_string());
        assert!(record.is_ok());
        
        let record = record.unwrap();
        assert_eq!(record.id, 1);
        assert_eq!(record.value, 500.0);
        assert_eq!(record.category, "Test");
    }
    
    #[test]
    fn test_invalid_id() {
        let record = DataRecord::new(0, 500.0, "Test".to_string());
        assert!(matches!(record, Err(ValidationError::InvalidId)));
    }
    
    #[test]
    fn test_normalize_value() {
        let record = DataRecord::new(1, 500.0, "Test".to_string()).unwrap();
        assert_eq!(record.normalize_value(), 0.5);
    }
    
    #[test]
    fn test_process_records() {
        let records = vec![
            DataRecord::new(1, 200.0, "alpha".to_string()).unwrap(),
            DataRecord::new(2, 800.0, "beta".to_string()).unwrap(),
        ];
        
        let processed = process_records(&records);
        assert_eq!(processed.len(), 2);
        assert_eq!(processed[0].1, 0.2);
        assert_eq!(processed[1].2, "BETA");
    }
}