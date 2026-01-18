
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub metadata: HashMap<String, String>,
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
    validation_rules: Vec<Box<dyn Fn(&DataRecord) -> Result<(), ProcessingError>>>,
    transformation_functions: Vec<Box<dyn Fn(DataRecord) -> Result<DataRecord, ProcessingError>>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            validation_rules: Vec::new(),
            transformation_functions: Vec::new(),
        }
    }

    pub fn add_validation_rule<F>(&mut self, rule: F)
    where
        F: Fn(&DataRecord) -> Result<(), ProcessingError> + 'static,
    {
        self.validation_rules.push(Box::new(rule));
    }

    pub fn add_transformation<F>(&mut self, transform: F)
    where
        F: Fn(DataRecord) -> Result<DataRecord, ProcessingError> + 'static,
    {
        self.transformation_functions.push(Box::new(transform));
    }

    pub fn process_record(&self, mut record: DataRecord) -> Result<DataRecord, ProcessingError> {
        for rule in &self.validation_rules {
            rule(&record)?;
        }

        for transform in &self.transformation_functions {
            record = transform(record)?;
        }

        Ok(record)
    }

    pub fn process_batch(&self, records: Vec<DataRecord>) -> Result<Vec<DataRecord>, ProcessingError> {
        let mut processed_records = Vec::with_capacity(records.len());
        
        for record in records {
            match self.process_record(record) {
                Ok(processed) => processed_records.push(processed),
                Err(e) => return Err(e),
            }
        }
        
        Ok(processed_records)
    }
}

pub fn create_default_processor() -> DataProcessor {
    let mut processor = DataProcessor::new();
    
    processor.add_validation_rule(|record| {
        if record.name.is_empty() {
            Err(ProcessingError::ValidationError("Name cannot be empty".to_string()))
        } else {
            Ok(())
        }
    });
    
    processor.add_validation_rule(|record| {
        if record.value < 0.0 {
            Err(ProcessingError::ValidationError("Value cannot be negative".to_string()))
        } else {
            Ok(())
        }
    });
    
    processor.add_transformation(|mut record| {
        record.name = record.name.trim().to_string();
        if record.name.is_empty() {
            Err(ProcessingError::TransformationFailed("Name became empty after trimming".to_string()))
        } else {
            Ok(record)
        }
    });
    
    processor.add_transformation(|mut record| {
        record.value = (record.value * 100.0).round() / 100.0;
        Ok(record)
    });
    
    processor
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_data_processing() {
        let processor = create_default_processor();
        
        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), "test".to_string());
        
        let record = DataRecord {
            id: 1,
            name: "  Test Record  ".to_string(),
            value: 123.456789,
            metadata,
        };
        
        let result = processor.process_record(record);
        assert!(result.is_ok());
        
        let processed = result.unwrap();
        assert_eq!(processed.name, "Test Record");
        assert_eq!(processed.value, 123.46);
    }
    
    #[test]
    fn test_validation_failure() {
        let processor = create_default_processor();
        
        let record = DataRecord {
            id: 2,
            name: "".to_string(),
            value: 50.0,
            metadata: HashMap::new(),
        };
        
        let result = processor.process_record(record);
        assert!(result.is_err());
    }
}
use std::collections::HashMap;
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

impl ValidationError {
    pub fn new(msg: &str) -> Self {
        ValidationError {
            message: msg.to_string(),
        }
    }
}

pub struct DataProcessor {
    data: HashMap<String, Vec<f64>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            data: HashMap::new(),
        }
    }

    pub fn add_dataset(&mut self, key: &str, values: Vec<f64>) -> Result<(), ValidationError> {
        if values.is_empty() {
            return Err(ValidationError::new("Dataset cannot be empty"));
        }

        for value in &values {
            if value.is_nan() || value.is_infinite() {
                return Err(ValidationError::new("Dataset contains invalid numeric values"));
            }
        }

        self.data.insert(key.to_string(), values);
        Ok(())
    }

    pub fn calculate_statistics(&self, key: &str) -> Result<Statistics, ValidationError> {
        let values = self.data.get(key)
            .ok_or_else(|| ValidationError::new("Dataset not found"))?;

        if values.is_empty() {
            return Err(ValidationError::new("Dataset is empty"));
        }

        let count = values.len();
        let sum: f64 = values.iter().sum();
        let mean = sum / count as f64;
        
        let variance: f64 = values.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / count as f64;
        
        let std_dev = variance.sqrt();
        
        let min = values.iter()
            .fold(f64::INFINITY, |a, &b| a.min(b));
        
        let max = values.iter()
            .fold(f64::NEG_INFINITY, |a, &b| a.max(b));

        Ok(Statistics {
            count,
            mean,
            std_dev,
            min,
            max,
            sum,
        })
    }

    pub fn normalize_data(&self, key: &str) -> Result<Vec<f64>, ValidationError> {
        let stats = self.calculate_statistics(key)?;
        
        if stats.std_dev == 0.0 {
            return Err(ValidationError::new("Cannot normalize data with zero standard deviation"));
        }

        let values = self.data.get(key).unwrap();
        let normalized: Vec<f64> = values.iter()
            .map(|&x| (x - stats.mean) / stats.std_dev)
            .collect();

        Ok(normalized)
    }

    pub fn merge_datasets(&mut self, key1: &str, key2: &str, new_key: &str) -> Result<(), ValidationError> {
        let values1 = self.data.get(key1)
            .ok_or_else(|| ValidationError::new("First dataset not found"))?;
        
        let values2 = self.data.get(key2)
            .ok_or_else(|| ValidationError::new("Second dataset not found"))?;

        let mut merged = values1.clone();
        merged.extend(values2.iter());

        self.add_dataset(new_key, merged)
    }

    pub fn list_datasets(&self) -> Vec<String> {
        self.data.keys().cloned().collect()
    }
}

#[derive(Debug, Clone)]
pub struct Statistics {
    pub count: usize,
    pub mean: f64,
    pub std_dev: f64,
    pub min: f64,
    pub max: f64,
    pub sum: f64,
}

impl fmt::Display for Statistics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Statistics: count={}, mean={:.4}, std_dev={:.4}, min={:.4}, max={:.4}, sum={:.4}",
            self.count, self.mean, self.std_dev, self.min, self.max, self.sum
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_calculate_statistics() {
        let mut processor = DataProcessor::new();
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        
        processor.add_dataset("test_data", data).unwrap();
        let stats = processor.calculate_statistics("test_data").unwrap();
        
        assert_eq!(stats.count, 5);
        assert_eq!(stats.mean, 3.0);
        assert_eq!(stats.sum, 15.0);
        assert_eq!(stats.min, 1.0);
        assert_eq!(stats.max, 5.0);
    }

    #[test]
    fn test_normalize_data() {
        let mut processor = DataProcessor::new();
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        
        processor.add_dataset("test_data", data).unwrap();
        let normalized = processor.normalize_data("test_data").unwrap();
        
        assert_eq!(normalized.len(), 5);
        let mean_normalized: f64 = normalized.iter().sum::<f64>() / normalized.len() as f64;
        assert!(mean_normalized.abs() < 1e-10);
    }

    #[test]
    fn test_merge_datasets() {
        let mut processor = DataProcessor::new();
        
        processor.add_dataset("set1", vec![1.0, 2.0]).unwrap();
        processor.add_dataset("set2", vec![3.0, 4.0]).unwrap();
        
        processor.merge_datasets("set1", "set2", "merged").unwrap();
        let stats = processor.calculate_statistics("merged").unwrap();
        
        assert_eq!(stats.count, 4);
        assert_eq!(stats.sum, 10.0);
    }
}