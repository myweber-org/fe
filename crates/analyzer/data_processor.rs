
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
    TransformationFailed(String),
    ValidationError(String),
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessingError::InvalidData(msg) => write!(f, "Invalid data: {}", msg),
            ProcessingError::TransformationFailed(msg) => write!(f, "Transformation failed: {}", msg),
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

    pub fn process_records(&mut self) -> Result<(), ProcessingError> {
        if self.records.is_empty() {
            return Err(ProcessingError::InvalidData("No records to process".to_string()));
        }

        self.calculate_statistics();
        self.normalize_values()?;
        self.validate_processed_data()?;
        
        Ok(())
    }

    pub fn get_category_stats(&self, category: &str) -> Option<&CategoryStats> {
        self.category_stats.get(category)
    }

    pub fn get_all_stats(&self) -> &HashMap<String, CategoryStats> {
        &self.category_stats
    }

    pub fn filter_by_threshold(&self, threshold: f64) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.value >= threshold)
            .collect()
    }

    fn validate_record(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if record.name.trim().is_empty() {
            return Err(ProcessingError::ValidationError(
                "Record name cannot be empty".to_string(),
            ));
        }

        if record.value.is_nan() || record.value.is_infinite() {
            return Err(ProcessingError::ValidationError(
                "Record value must be a valid number".to_string(),
            ));
        }

        if record.category.trim().is_empty() {
            return Err(ProcessingError::ValidationError(
                "Category cannot be empty".to_string(),
            ));
        }

        Ok(())
    }

    fn update_category_stats(&mut self, record: &DataRecord) {
        let stats = self.category_stats
            .entry(record.category.clone())
            .or_insert(CategoryStats {
                count: 0,
                total_value: 0.0,
                average_value: 0.0,
            });

        stats.count += 1;
        stats.total_value += record.value;
    }

    fn calculate_statistics(&mut self) {
        for (_, stats) in self.category_stats.iter_mut() {
            if stats.count > 0 {
                stats.average_value = stats.total_value / stats.count as f64;
            }
        }
    }

    fn normalize_values(&mut self) -> Result<(), ProcessingError> {
        if self.records.is_empty() {
            return Ok(());
        }

        let max_value = self.records
            .iter()
            .map(|r| r.value)
            .fold(f64::NEG_INFINITY, f64::max);

        if max_value <= 0.0 {
            return Err(ProcessingError::TransformationFailed(
                "Cannot normalize with non-positive maximum value".to_string(),
            ));
        }

        for record in &mut self.records {
            record.value = record.value / max_value;
        }

        Ok(())
    }

    fn validate_processed_data(&self) -> Result<(), ProcessingError> {
        for record in &self.records {
            if record.value < 0.0 || record.value > 1.0 {
                return Err(ProcessingError::ValidationError(
                    format!("Normalized value out of range: {}", record.value),
                ));
            }
        }
        Ok(())
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
            name: "Test Record".to_string(),
            value: 42.5,
            category: "Test".to_string(),
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
            value: 42.5,
            category: "Test".to_string(),
        };

        assert!(processor.add_record(record).is_err());
    }

    #[test]
    fn test_category_stats() {
        let mut processor = DataProcessor::new();
        
        let record1 = DataRecord {
            id: 1,
            name: "Record 1".to_string(),
            value: 10.0,
            category: "CategoryA".to_string(),
        };

        let record2 = DataRecord {
            id: 2,
            name: "Record 2".to_string(),
            value: 20.0,
            category: "CategoryA".to_string(),
        };

        let record3 = DataRecord {
            id: 3,
            name: "Record 3".to_string(),
            value: 30.0,
            category: "CategoryB".to_string(),
        };

        processor.add_record(record1).unwrap();
        processor.add_record(record2).unwrap();
        processor.add_record(record3).unwrap();
        processor.process_records().unwrap();

        let stats_a = processor.get_category_stats("CategoryA").unwrap();
        assert_eq!(stats_a.count, 2);
        assert_eq!(stats_a.total_value, 30.0);
        assert_eq!(stats_a.average_value, 15.0);

        let stats_b = processor.get_category_stats("CategoryB").unwrap();
        assert_eq!(stats_b.count, 1);
        assert_eq!(stats_b.total_value, 30.0);
        assert_eq!(stats_b.average_value, 30.0);
    }

    #[test]
    fn test_filter_by_threshold() {
        let mut processor = DataProcessor::new();
        
        for i in 0..10 {
            let record = DataRecord {
                id: i,
                name: format!("Record {}", i),
                value: i as f64 * 10.0,
                category: "Test".to_string(),
            };
            processor.add_record(record).unwrap();
        }

        let filtered = processor.filter_by_threshold(50.0);
        assert_eq!(filtered.len(), 5);
    }
}
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct ValidationError {
    message: String,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Validation error: {}", self.message)
    }
}

impl Error for ValidationError {}

pub struct DataProcessor {
    threshold: f64,
}

impl DataProcessor {
    pub fn new(threshold: f64) -> Result<Self, ValidationError> {
        if threshold < 0.0 || threshold > 1.0 {
            return Err(ValidationError {
                message: format!("Threshold {} must be between 0.0 and 1.0", threshold),
            });
        }
        
        Ok(DataProcessor { threshold })
    }
    
    pub fn process_data(&self, input: &[f64]) -> Result<Vec<f64>, ValidationError> {
        if input.is_empty() {
            return Err(ValidationError {
                message: "Input data cannot be empty".to_string(),
            });
        }
        
        let mean = input.iter().sum::<f64>() / input.len() as f64;
        
        let result: Vec<f64> = input
            .iter()
            .map(|&value| {
                if value > mean * self.threshold {
                    value * 2.0
                } else {
                    value / 2.0
                }
            })
            .collect();
        
        Ok(result)
    }
    
    pub fn calculate_statistics(&self, data: &[f64]) -> (f64, f64, f64) {
        let count = data.len() as f64;
        let sum: f64 = data.iter().sum();
        let mean = sum / count;
        
        let variance: f64 = data
            .iter()
            .map(|&value| {
                let diff = value - mean;
                diff * diff
            })
            .sum::<f64>() / count;
        
        let std_dev = variance.sqrt();
        
        (mean, variance, std_dev)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_processor_creation() {
        let processor = DataProcessor::new(0.5);
        assert!(processor.is_ok());
        
        let invalid = DataProcessor::new(1.5);
        assert!(invalid.is_err());
    }
    
    #[test]
    fn test_data_processing() {
        let processor = DataProcessor::new(0.5).unwrap();
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        
        let result = processor.process_data(&data);
        assert!(result.is_ok());
        
        let processed = result.unwrap();
        assert_eq!(processed.len(), data.len());
    }
    
    #[test]
    fn test_statistics_calculation() {
        let processor = DataProcessor::new(0.7).unwrap();
        let data = vec![2.0, 4.0, 6.0, 8.0, 10.0];
        
        let (mean, variance, std_dev) = processor.calculate_statistics(&data);
        
        assert_eq!(mean, 6.0);
        assert_eq!(variance, 8.0);
        assert!((std_dev - 2.8284271247461903).abs() < 1e-10);
    }
}