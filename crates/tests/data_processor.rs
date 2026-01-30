
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

#[derive(Debug)]
pub enum ValidationError {
    InvalidId,
    EmptyName,
    NegativeValue,
    UnknownCategory,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::InvalidId => write!(f, "ID must be positive integer"),
            ValidationError::EmptyName => write!(f, "Name cannot be empty"),
            ValidationError::NegativeValue => write!(f, "Value must be non-negative"),
            ValidationError::UnknownCategory => write!(f, "Category not recognized"),
        }
    }
}

impl Error for ValidationError {}

pub struct DataProcessor {
    valid_categories: Vec<String>,
    transformation_rules: HashMap<String, f64>,
}

impl DataProcessor {
    pub fn new() -> Self {
        let mut transformation_rules = HashMap::new();
        transformation_rules.insert("standard".to_string(), 1.0);
        transformation_rules.insert("premium".to_string(), 1.5);
        transformation_rules.insert("economy".to_string(), 0.8);
        
        DataProcessor {
            valid_categories: vec![
                "standard".to_string(),
                "premium".to_string(),
                "economy".to_string(),
            ],
            transformation_rules,
        }
    }
    
    pub fn validate_record(&self, record: &DataRecord) -> Result<(), ValidationError> {
        if record.id == 0 {
            return Err(ValidationError::InvalidId);
        }
        
        if record.name.trim().is_empty() {
            return Err(ValidationError::EmptyName);
        }
        
        if record.value < 0.0 {
            return Err(ValidationError::NegativeValue);
        }
        
        if !self.valid_categories.contains(&record.category) {
            return Err(ValidationError::UnknownCategory);
        }
        
        Ok(())
    }
    
    pub fn transform_value(&self, record: &DataRecord) -> f64 {
        match self.transformation_rules.get(&record.category) {
            Some(factor) => record.value * factor,
            None => record.value,
        }
    }
    
    pub fn process_batch(&self, records: Vec<DataRecord>) -> Vec<Result<f64, ValidationError>> {
        records
            .iter()
            .map(|record| {
                self.validate_record(record)
                    .map(|_| self.transform_value(record))
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_valid_record() {
        let processor = DataProcessor::new();
        let record = DataRecord {
            id: 1,
            name: "Test Item".to_string(),
            value: 100.0,
            category: "standard".to_string(),
        };
        
        assert!(processor.validate_record(&record).is_ok());
        assert_eq!(processor.transform_value(&record), 100.0);
    }
    
    #[test]
    fn test_invalid_category() {
        let processor = DataProcessor::new();
        let record = DataRecord {
            id: 1,
            name: "Test Item".to_string(),
            value: 100.0,
            category: "invalid".to_string(),
        };
        
        assert!(matches!(
            processor.validate_record(&record),
            Err(ValidationError::UnknownCategory)
        ));
    }
    
    #[test]
    fn test_premium_transformation() {
        let processor = DataProcessor::new();
        let record = DataRecord {
            id: 2,
            name: "Premium Item".to_string(),
            value: 100.0,
            category: "premium".to_string(),
        };
        
        assert_eq!(processor.transform_value(&record), 150.0);
    }
}use csv::Reader;
use serde::Deserialize;
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize)]
struct Record {
    id: u32,
    value: f64,
    category: String,
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

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let mut rdr = Reader::from_reader(file);
        
        for result in rdr.deserialize() {
            let record: Record = result?;
            self.records.push(record);
        }
        
        Ok(())
    }

    pub fn calculate_statistics(&self) -> (f64, f64, f64) {
        if self.records.is_empty() {
            return (0.0, 0.0, 0.0);
        }

        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        let count = self.records.len() as f64;
        let mean = sum / count;

        let variance: f64 = self.records
            .iter()
            .map(|r| (r.value - mean).powi(2))
            .sum::<f64>() / count;

        let std_dev = variance.sqrt();

        (mean, variance, std_dev)
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|r| r.category == category)
            .collect()
    }

    pub fn get_record_count(&self) -> usize {
        self.records.len()
    }

    pub fn get_max_value(&self) -> Option<f64> {
        self.records.iter().map(|r| r.value).reduce(f64::max)
    }

    pub fn get_min_value(&self) -> Option<f64> {
        self.records.iter().map(|r| r.value).reduce(f64::min)
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
        writeln!(temp_file, "1,10.5,category_a").unwrap();
        writeln!(temp_file, "2,20.3,category_b").unwrap();
        writeln!(temp_file, "3,15.7,category_a").unwrap();
        
        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(processor.get_record_count(), 3);
        
        let stats = processor.calculate_statistics();
        assert!((stats.0 - 15.5).abs() < 0.1);
        
        let filtered = processor.filter_by_category("category_a");
        assert_eq!(filtered.len(), 2);
        
        assert_eq!(processor.get_max_value(), Some(20.3));
        assert_eq!(processor.get_min_value(), Some(10.5));
    }
}