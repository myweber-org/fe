
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, PartialEq)]
pub struct DataRecord {
    id: u32,
    value: f64,
    category: String,
    valid: bool,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: String) -> Self {
        let valid = value >= 0.0 && value <= 1000.0;
        DataRecord {
            id,
            value,
            category,
            valid,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.valid
    }

    pub fn get_value(&self) -> f64 {
        self.value
    }
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
    total_value: f64,
    valid_count: usize,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
            total_value: 0.0,
            valid_count: 0,
        }
    }

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            if line_num == 0 {
                continue;
            }

            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() != 3 {
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

            let record = DataRecord::new(id, value, category);
            self.add_record(record);
        }

        Ok(())
    }

    pub fn add_record(&mut self, record: DataRecord) {
        if record.is_valid() {
            self.total_value += record.get_value();
            self.valid_count += 1;
        }
        self.records.push(record);
    }

    pub fn get_average_value(&self) -> Option<f64> {
        if self.valid_count > 0 {
            Some(self.total_value / self.valid_count as f64)
        } else {
            None
        }
    }

    pub fn get_valid_records(&self) -> Vec<&DataRecord> {
        self.records.iter().filter(|r| r.is_valid()).collect()
    }

    pub fn count_by_category(&self) -> std::collections::HashMap<String, usize> {
        let mut counts = std::collections::HashMap::new();
        for record in &self.records {
            if record.is_valid() {
                *counts.entry(record.category.clone()).or_insert(0) += 1;
            }
        }
        counts
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_validation() {
        let valid_record = DataRecord::new(1, 500.0, "A".to_string());
        assert!(valid_record.is_valid());

        let invalid_record = DataRecord::new(2, -10.0, "B".to_string());
        assert!(!invalid_record.is_valid());
    }

    #[test]
    fn test_processor_average() {
        let mut processor = DataProcessor::new();
        processor.add_record(DataRecord::new(1, 100.0, "Test".to_string()));
        processor.add_record(DataRecord::new(2, 200.0, "Test".to_string()));
        processor.add_record(DataRecord::new(3, -50.0, "Test".to_string()));

        assert_eq!(processor.get_average_value(), Some(150.0));
        assert_eq!(processor.valid_count, 2);
    }
}
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct ProcessingError {
    details: String,
}

impl ProcessingError {
    pub fn new(msg: &str) -> Self {
        ProcessingError {
            details: msg.to_string(),
        }
    }
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl Error for ProcessingError {}

pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub timestamp: i64,
}

impl DataRecord {
    pub fn validate(&self) -> Result<(), ProcessingError> {
        if self.id == 0 {
            return Err(ProcessingError::new("ID cannot be zero"));
        }
        
        if self.value < 0.0 || self.value > 1000.0 {
            return Err(ProcessingError::new("Value must be between 0 and 1000"));
        }
        
        if self.timestamp < 0 {
            return Err(ProcessingError::new("Timestamp cannot be negative"));
        }
        
        Ok(())
    }
    
    pub fn transform(&mut self, multiplier: f64) -> Result<(), ProcessingError> {
        if multiplier <= 0.0 {
            return Err(ProcessingError::new("Multiplier must be positive"));
        }
        
        self.value *= multiplier;
        Ok(())
    }
}

pub fn process_records(records: &mut [DataRecord], multiplier: f64) -> Result<Vec<DataRecord>, ProcessingError> {
    let mut processed = Vec::with_capacity(records.len());
    
    for record in records.iter_mut() {
        record.validate()?;
        record.transform(multiplier)?;
        processed.push(DataRecord {
            id: record.id,
            value: record.value,
            timestamp: record.timestamp,
        });
    }
    
    Ok(processed)
}

pub fn calculate_statistics(records: &[DataRecord]) -> (f64, f64, f64) {
    if records.is_empty() {
        return (0.0, 0.0, 0.0);
    }
    
    let sum: f64 = records.iter().map(|r| r.value).sum();
    let count = records.len() as f64;
    let mean = sum / count;
    
    let variance: f64 = records.iter()
        .map(|r| (r.value - mean).powi(2))
        .sum::<f64>() / count;
    
    let std_dev = variance.sqrt();
    
    (mean, variance, std_dev)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_record_validation() {
        let valid_record = DataRecord { id: 1, value: 500.0, timestamp: 1234567890 };
        assert!(valid_record.validate().is_ok());
        
        let invalid_record = DataRecord { id: 0, value: 500.0, timestamp: 1234567890 };
        assert!(invalid_record.validate().is_err());
    }
    
    #[test]
    fn test_record_transformation() {
        let mut record = DataRecord { id: 1, value: 100.0, timestamp: 1234567890 };
        assert!(record.transform(2.0).is_ok());
        assert_eq!(record.value, 200.0);
        
        assert!(record.transform(0.0).is_err());
    }
    
    #[test]
    fn test_process_records() {
        let mut records = vec![
            DataRecord { id: 1, value: 100.0, timestamp: 1234567890 },
            DataRecord { id: 2, value: 200.0, timestamp: 1234567891 },
        ];
        
        let result = process_records(&mut records, 1.5);
        assert!(result.is_ok());
        let processed = result.unwrap();
        assert_eq!(processed[0].value, 150.0);
        assert_eq!(processed[1].value, 300.0);
    }
    
    #[test]
    fn test_calculate_statistics() {
        let records = vec![
            DataRecord { id: 1, value: 10.0, timestamp: 1234567890 },
            DataRecord { id: 2, value: 20.0, timestamp: 1234567891 },
            DataRecord { id: 3, value: 30.0, timestamp: 1234567892 },
        ];
        
        let (mean, variance, std_dev) = calculate_statistics(&records);
        assert_eq!(mean, 20.0);
        assert_eq!(variance, 66.66666666666667);
        assert_eq!(std_dev, 8.16496580927726);
    }
}
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub category: String,
}

#[derive(Debug)]
pub enum ProcessingError {
    InvalidData(String),
    TransformationError(String),
    ValidationError(String),
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessingError::InvalidData(msg) => write!(f, "Invalid data: {}", msg),
            ProcessingError::TransformationError(msg) => write!(f, "Transformation error: {}", msg),
            ProcessingError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl Error for ProcessingError {}

pub struct DataProcessor {
    records: Vec<DataRecord>,
    category_stats: HashMap<String, CategoryStats>,
}

#[derive(Debug, Clone)]
pub struct CategoryStats {
    pub category: String,
    pub count: usize,
    pub total_value: f64,
    pub average_value: f64,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
            category_stats: HashMap::new(),
        }
    }

    pub fn add_record(&mut self, record: DataRecord) -> Result<(), ProcessingError> {
        self.validate_record(&record)?;
        self.records.push(record.clone());
        self.update_category_stats(&record);
        Ok(())
    }

    pub fn process_records(&mut self) -> Result<Vec<DataRecord>, ProcessingError> {
        if self.records.is_empty() {
            return Err(ProcessingError::InvalidData("No records to process".to_string()));
        }

        let processed_records: Vec<DataRecord> = self.records
            .iter()
            .map(|record| self.transform_record(record))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(processed_records)
    }

    pub fn get_category_stats(&self) -> &HashMap<String, CategoryStats> {
        &self.category_stats
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    fn validate_record(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if record.name.trim().is_empty() {
            return Err(ProcessingError::ValidationError("Name cannot be empty".to_string()));
        }

        if record.value < 0.0 {
            return Err(ProcessingError::ValidationError("Value cannot be negative".to_string()));
        }

        if record.category.trim().is_empty() {
            return Err(ProcessingError::ValidationError("Category cannot be empty".to_string()));
        }

        Ok(())
    }

    fn transform_record(&self, record: &DataRecord) -> Result<DataRecord, ProcessingError> {
        let transformed_value = if record.value > 1000.0 {
            record.value * 0.9
        } else {
            record.value
        };

        let transformed_name = format!("PROCESSED_{}", record.name.to_uppercase());

        Ok(DataRecord {
            id: record.id,
            name: transformed_name,
            value: transformed_value,
            category: record.category.clone(),
        })
    }

    fn update_category_stats(&mut self, record: &DataRecord) {
        let stats = self.category_stats
            .entry(record.category.clone())
            .or_insert(CategoryStats {
                category: record.category.clone(),
                count: 0,
                total_value: 0.0,
                average_value: 0.0,
            });

        stats.count += 1;
        stats.total_value += record.value;
        stats.average_value = stats.total_value / stats.count as f64;
    }
}

impl Default for DataProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_valid_record() {
        let mut processor = DataProcessor::new();
        let record = DataRecord {
            id: 1,
            name: "Test".to_string(),
            value: 100.0,
            category: "A".to_string(),
        };

        assert!(processor.add_record(record).is_ok());
        assert_eq!(processor.records.len(), 1);
    }

    #[test]
    fn test_add_invalid_record() {
        let mut processor = DataProcessor::new();
        let record = DataRecord {
            id: 1,
            name: "".to_string(),
            value: 100.0,
            category: "A".to_string(),
        };

        assert!(processor.add_record(record).is_err());
    }

    #[test]
    fn test_process_records() {
        let mut processor = DataProcessor::new();
        
        let records = vec![
            DataRecord {
                id: 1,
                name: "test1".to_string(),
                value: 500.0,
                category: "A".to_string(),
            },
            DataRecord {
                id: 2,
                name: "test2".to_string(),
                value: 1500.0,
                category: "B".to_string(),
            },
        ];

        for record in records {
            processor.add_record(record).unwrap();
        }

        let processed = processor.process_records().unwrap();
        assert_eq!(processed.len(), 2);
        assert!(processed[0].name.starts_with("PROCESSED_"));
        assert_eq!(processed[1].value, 1350.0);
    }

    #[test]
    fn test_category_stats() {
        let mut processor = DataProcessor::new();
        
        let records = vec![
            DataRecord {
                id: 1,
                name: "test1".to_string(),
                value: 100.0,
                category: "A".to_string(),
            },
            DataRecord {
                id: 2,
                name: "test2".to_string(),
                value: 200.0,
                category: "A".to_string(),
            },
        ];

        for record in records {
            processor.add_record(record).unwrap();
        }

        let stats = processor.get_category_stats();
        assert!(stats.contains_key("A"));
        let category_a_stats = stats.get("A").unwrap();
        assert_eq!(category_a_stats.count, 2);
        assert_eq!(category_a_stats.average_value, 150.0);
    }
}