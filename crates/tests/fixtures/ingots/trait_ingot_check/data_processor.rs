
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
    pub total_value: f64,
    pub record_count: usize,
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

        self.calculate_category_stats();
        Ok(())
    }

    pub fn get_category_stats(&self, category: &str) -> Option<&CategoryStats> {
        self.category_stats.get(category)
    }

    pub fn transform_values<F>(&mut self, transform_fn: F) -> Result<(), ProcessingError>
    where
        F: Fn(f64) -> f64,
    {
        for record in &mut self.records {
            let original_value = record.value;
            record.value = transform_fn(original_value);
            
            if record.value.is_nan() || record.value.is_infinite() {
                return Err(ProcessingError::TransformationError(
                    format!("Transformation produced invalid value for record {}", record.id)
                ));
            }
        }
        
        self.calculate_category_stats();
        Ok(())
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    pub fn total_records(&self) -> usize {
        self.records.len()
    }

    pub fn average_value(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }
        
        let total: f64 = self.records.iter().map(|r| r.value).sum();
        Some(total / self.records.len() as f64)
    }

    fn validate_record(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if record.name.trim().is_empty() {
            return Err(ProcessingError::ValidationError(
                format!("Record {} has empty name", record.id)
            ));
        }
        
        if record.value < 0.0 {
            return Err(ProcessingError::ValidationError(
                format!("Record {} has negative value: {}", record.id, record.value)
            ));
        }
        
        if record.category.trim().is_empty() {
            return Err(ProcessingError::ValidationError(
                format!("Record {} has empty category", record.id)
            ));
        }
        
        Ok(())
    }

    fn update_category_stats(&mut self, record: &DataRecord) {
        let stats = self.category_stats
            .entry(record.category.clone())
            .or_insert(CategoryStats {
                total_value: 0.0,
                record_count: 0,
                average_value: 0.0,
            });
        
        stats.total_value += record.value;
        stats.record_count += 1;
        stats.average_value = stats.total_value / stats.record_count as f64;
    }

    fn calculate_category_stats(&mut self) {
        self.category_stats.clear();
        
        for record in &self.records {
            self.update_category_stats(record);
        }
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
            value: 100.0,
            category: "Test".to_string(),
        };
        
        assert!(processor.add_record(record).is_ok());
        assert_eq!(processor.total_records(), 1);
    }

    #[test]
    fn test_add_invalid_record() {
        let mut processor = DataProcessor::new();
        let record = DataRecord {
            id: 1,
            name: "".to_string(),
            value: 100.0,
            category: "Test".to_string(),
        };
        
        assert!(processor.add_record(record).is_err());
        assert_eq!(processor.total_records(), 0);
    }

    #[test]
    fn test_transform_values() {
        let mut processor = DataProcessor::new();
        
        let record1 = DataRecord {
            id: 1,
            name: "Record 1".to_string(),
            value: 10.0,
            category: "A".to_string(),
        };
        
        let record2 = DataRecord {
            id: 2,
            name: "Record 2".to_string(),
            value: 20.0,
            category: "B".to_string(),
        };
        
        processor.add_record(record1).unwrap();
        processor.add_record(record2).unwrap();
        
        let transform_result = processor.transform_values(|x| x * 2.0);
        assert!(transform_result.is_ok());
        
        let stats_a = processor.get_category_stats("A").unwrap();
        assert_eq!(stats_a.total_value, 20.0);
        
        let stats_b = processor.get_category_stats("B").unwrap();
        assert_eq!(stats_b.total_value, 40.0);
    }

    #[test]
    fn test_filter_by_category() {
        let mut processor = DataProcessor::new();
        
        let record1 = DataRecord {
            id: 1,
            name: "Record 1".to_string(),
            value: 10.0,
            category: "A".to_string(),
        };
        
        let record2 = DataRecord {
            id: 2,
            name: "Record 2".to_string(),
            value: 20.0,
            category: "B".to_string(),
        };
        
        let record3 = DataRecord {
            id: 3,
            name: "Record 3".to_string(),
            value: 30.0,
            category: "A".to_string(),
        };
        
        processor.add_record(record1).unwrap();
        processor.add_record(record2).unwrap();
        processor.add_record(record3).unwrap();
        
        let category_a_records = processor.filter_by_category("A");
        assert_eq!(category_a_records.len(), 2);
        
        let category_b_records = processor.filter_by_category("B");
        assert_eq!(category_b_records.len(), 1);
    }

    #[test]
    fn test_average_value() {
        let mut processor = DataProcessor::new();
        
        assert!(processor.average_value().is_none());
        
        let record1 = DataRecord {
            id: 1,
            name: "Record 1".to_string(),
            value: 10.0,
            category: "A".to_string(),
        };
        
        let record2 = DataRecord {
            id: 2,
            name: "Record 2".to_string(),
            value: 20.0,
            category: "B".to_string(),
        };
        
        processor.add_record(record1).unwrap();
        processor.add_record(record2).unwrap();
        
        assert_eq!(processor.average_value(), Some(15.0));
    }
}