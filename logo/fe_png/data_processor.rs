
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
pub enum ValidationError {
    InvalidId,
    EmptyName,
    InvalidValue,
    UnknownCategory,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::InvalidId => write!(f, "ID must be greater than 0"),
            ValidationError::EmptyName => write!(f, "Name cannot be empty"),
            ValidationError::InvalidValue => write!(f, "Value must be between 0.0 and 1000.0"),
            ValidationError::UnknownCategory => write!(f, "Category is not recognized"),
        }
    }
}

impl Error for ValidationError {}

pub struct DataProcessor {
    valid_categories: HashMap<String, bool>,
}

impl DataProcessor {
    pub fn new() -> Self {
        let mut categories = HashMap::new();
        categories.insert("A".to_string(), true);
        categories.insert("B".to_string(), true);
        categories.insert("C".to_string(), true);
        
        DataProcessor {
            valid_categories: categories,
        }
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), ValidationError> {
        if record.id == 0 {
            return Err(ValidationError::InvalidId);
        }
        
        if record.name.trim().is_empty() {
            return Err(ValidationError::EmptyName);
        }
        
        if record.value < 0.0 || record.value > 1000.0 {
            return Err(ValidationError::InvalidValue);
        }
        
        if !self.valid_categories.contains_key(&record.category) {
            return Err(ValidationError::UnknownCategory);
        }
        
        Ok(())
    }

    pub fn process_records(&self, records: Vec<DataRecord>) -> Vec<Result<DataRecord, ValidationError>> {
        records
            .into_iter()
            .map(|record| {
                self.validate_record(&record)
                    .map(|_| {
                        let processed_record = DataRecord {
                            value: (record.value * 100.0).round() / 100.0,
                            ..record
                        };
                        processed_record
                    })
            })
            .collect()
    }

    pub fn calculate_statistics(&self, records: &[DataRecord]) -> (f64, f64, f64) {
        if records.is_empty() {
            return (0.0, 0.0, 0.0);
        }

        let sum: f64 = records.iter().map(|r| r.value).sum();
        let count = records.len() as f64;
        let average = sum / count;
        
        let variance: f64 = records
            .iter()
            .map(|r| (r.value - average).powi(2))
            .sum::<f64>() / count;
        
        let std_dev = variance.sqrt();
        
        (average, variance, std_dev)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_success() {
        let processor = DataProcessor::new();
        let record = DataRecord {
            id: 1,
            name: "Test".to_string(),
            value: 50.0,
            category: "A".to_string(),
        };
        
        assert!(processor.validate_record(&record).is_ok());
    }

    #[test]
    fn test_validation_failure() {
        let processor = DataProcessor::new();
        let record = DataRecord {
            id: 0,
            name: "".to_string(),
            value: -10.0,
            category: "X".to_string(),
        };
        
        assert!(processor.validate_record(&record).is_err());
    }

    #[test]
    fn test_process_records() {
        let processor = DataProcessor::new();
        let records = vec![
            DataRecord {
                id: 1,
                name: "Record1".to_string(),
                value: 123.456,
                category: "A".to_string(),
            },
            DataRecord {
                id: 2,
                name: "Record2".to_string(),
                value: 78.912,
                category: "B".to_string(),
            },
        ];
        
        let results = processor.process_records(records);
        assert_eq!(results.len(), 2);
        assert!(results[0].is_ok());
        assert!(results[1].is_ok());
        
        if let Ok(record) = &results[0] {
            assert_eq!(record.value, 123.46);
        }
    }

    #[test]
    fn test_calculate_statistics() {
        let processor = DataProcessor::new();
        let records = vec![
            DataRecord {
                id: 1,
                name: "R1".to_string(),
                value: 10.0,
                category: "A".to_string(),
            },
            DataRecord {
                id: 2,
                name: "R2".to_string(),
                value: 20.0,
                category: "B".to_string(),
            },
            DataRecord {
                id: 3,
                name: "R3".to_string(),
                value: 30.0,
                category: "C".to_string(),
            },
        ];
        
        let (avg, var, std) = processor.calculate_statistics(&records);
        assert_eq!(avg, 20.0);
        assert_eq!(var, 66.66666666666667);
        assert_eq!(std, 8.16496580927726);
    }
}