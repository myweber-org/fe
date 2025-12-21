
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub category: String,
    pub timestamp: i64,
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
            let timestamp = match parts[3].parse::<i64>() {
                Ok(val) => val,
                Err(_) => continue,
            };

            self.records.push(DataRecord {
                id,
                value,
                category,
                timestamp,
            });
            count += 1;
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

    pub fn get_records_count(&self) -> usize {
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
        assert_eq!(processor.get_records_count(), 0);

        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category,timestamp").unwrap();
        writeln!(temp_file, "1,10.5,alpha,1000").unwrap();
        writeln!(temp_file, "2,20.3,beta,2000").unwrap();
        writeln!(temp_file, "3,15.7,alpha,3000").unwrap();

        let count = processor.load_from_csv(temp_file.path()).unwrap();
        assert_eq!(count, 3);
        assert_eq!(processor.get_records_count(), 3);

        let alpha_records = processor.filter_by_category("alpha");
        assert_eq!(alpha_records.len(), 2);

        let average = processor.calculate_average().unwrap();
        assert!((average - 15.5).abs() < 0.1);

        processor.clear();
        assert_eq!(processor.get_records_count(), 0);
    }
}
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub category: String,
}

#[derive(Debug)]
pub enum DataError {
    InvalidId,
    InvalidValue,
    EmptyCategory,
    TransformationError(String),
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataError::InvalidId => write!(f, "ID must be greater than zero"),
            DataError::InvalidValue => write!(f, "Value must be between 0.0 and 1000.0"),
            DataError::EmptyCategory => write!(f, "Category cannot be empty"),
            DataError::TransformationError(msg) => write!(f, "Transformation failed: {}", msg),
        }
    }
}

impl Error for DataError {}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: String) -> Result<Self, DataError> {
        if id == 0 {
            return Err(DataError::InvalidId);
        }
        if value < 0.0 || value > 1000.0 {
            return Err(DataError::InvalidValue);
        }
        if category.trim().is_empty() {
            return Err(DataError::EmptyCategory);
        }

        Ok(Self {
            id,
            value,
            category: category.trim().to_string(),
        })
    }

    pub fn transform(&self, multiplier: f64) -> Result<Self, DataError> {
        if multiplier <= 0.0 {
            return Err(DataError::TransformationError(
                "Multiplier must be positive".to_string(),
            ));
        }

        let new_value = self.value * multiplier;
        if new_value > 1000.0 {
            return Err(DataError::TransformationError(
                "Transformed value exceeds maximum limit".to_string(),
            ));
        }

        Ok(Self {
            id: self.id,
            value: new_value,
            category: self.category.clone(),
        })
    }

    pub fn normalize(&self, base_value: f64) -> f64 {
        if base_value == 0.0 {
            return 0.0;
        }
        self.value / base_value
    }
}

pub fn process_records(
    records: Vec<DataRecord>,
    multiplier: f64,
) -> Result<Vec<DataRecord>, DataError> {
    let mut processed = Vec::with_capacity(records.len());

    for record in records {
        let transformed = record.transform(multiplier)?;
        processed.push(transformed);
    }

    Ok(processed)
}

pub fn calculate_average(records: &[DataRecord]) -> Option<f64> {
    if records.is_empty() {
        return None;
    }

    let sum: f64 = records.iter().map(|r| r.value).sum();
    Some(sum / records.len() as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_record_creation() {
        let record = DataRecord::new(1, 100.0, "test".to_string());
        assert!(record.is_ok());
        let record = record.unwrap();
        assert_eq!(record.id, 1);
        assert_eq!(record.value, 100.0);
        assert_eq!(record.category, "test");
    }

    #[test]
    fn test_invalid_id() {
        let record = DataRecord::new(0, 100.0, "test".to_string());
        assert!(matches!(record, Err(DataError::InvalidId)));
    }

    #[test]
    fn test_invalid_value() {
        let record = DataRecord::new(1, -10.0, "test".to_string());
        assert!(matches!(record, Err(DataError::InvalidValue)));

        let record = DataRecord::new(1, 1500.0, "test".to_string());
        assert!(matches!(record, Err(DataError::InvalidValue)));
    }

    #[test]
    fn test_empty_category() {
        let record = DataRecord::new(1, 100.0, "".to_string());
        assert!(matches!(record, Err(DataError::EmptyCategory)));

        let record = DataRecord::new(1, 100.0, "   ".to_string());
        assert!(matches!(record, Err(DataError::EmptyCategory)));
    }

    #[test]
    fn test_transform_record() {
        let record = DataRecord::new(1, 100.0, "test".to_string()).unwrap();
        let transformed = record.transform(2.0).unwrap();
        assert_eq!(transformed.value, 200.0);
    }

    #[test]
    fn test_normalize() {
        let record = DataRecord::new(1, 50.0, "test".to_string()).unwrap();
        assert_eq!(record.normalize(100.0), 0.5);
        assert_eq!(record.normalize(0.0), 0.0);
    }

    #[test]
    fn test_process_records() {
        let records = vec![
            DataRecord::new(1, 100.0, "a".to_string()).unwrap(),
            DataRecord::new(2, 200.0, "b".to_string()).unwrap(),
        ];

        let processed = process_records(records, 1.5).unwrap();
        assert_eq!(processed[0].value, 150.0);
        assert_eq!(processed[1].value, 300.0);
    }

    #[test]
    fn test_calculate_average() {
        let records = vec![
            DataRecord::new(1, 100.0, "a".to_string()).unwrap(),
            DataRecord::new(2, 200.0, "b".to_string()).unwrap(),
            DataRecord::new(3, 300.0, "c".to_string()).unwrap(),
        ];

        let avg = calculate_average(&records);
        assert_eq!(avg, Some(200.0));

        let empty: Vec<DataRecord> = vec![];
        let avg = calculate_average(&empty);
        assert_eq!(avg, None);
    }
}