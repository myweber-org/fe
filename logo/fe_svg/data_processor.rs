
use std::collections::HashMap;

pub struct DataProcessor {
    cache: HashMap<String, Vec<f64>>,
    validation_rules: Vec<ValidationRule>,
}

#[derive(Debug, Clone)]
pub struct ValidationRule {
    pub min_value: f64,
    pub max_value: f64,
    pub required: bool,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            cache: HashMap::new(),
            validation_rules: vec![
                ValidationRule {
                    min_value: 0.0,
                    max_value: 100.0,
                    required: true,
                },
            ],
        }
    }

    pub fn process_dataset(&mut self, dataset_id: &str, data: &[f64]) -> Result<Vec<f64>, String> {
        if data.is_empty() {
            return Err("Dataset cannot be empty".to_string());
        }

        for value in data {
            for rule in &self.validation_rules {
                if rule.required && (*value < rule.min_value || *value > rule.max_value) {
                    return Err(format!(
                        "Value {} outside valid range [{}, {}]",
                        value, rule.min_value, rule.max_value
                    ));
                }
            }
        }

        let processed: Vec<f64> = data
            .iter()
            .map(|&x| {
                let normalized = (x - 50.0) / 50.0;
                normalized.clamp(-1.0, 1.0)
            })
            .collect();

        self.cache.insert(dataset_id.to_string(), processed.clone());
        Ok(processed)
    }

    pub fn get_cached_result(&self, dataset_id: &str) -> Option<&Vec<f64>> {
        self.cache.get(dataset_id)
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    pub fn add_validation_rule(&mut self, rule: ValidationRule) {
        self.validation_rules.push(rule);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_processing() {
        let mut processor = DataProcessor::new();
        let data = vec![25.0, 50.0, 75.0];
        
        let result = processor.process_dataset("test1", &data);
        assert!(result.is_ok());
        
        let processed = result.unwrap();
        assert_eq!(processed.len(), 3);
        assert!(processed[0] >= -1.0 && processed[0] <= 1.0);
    }

    #[test]
    fn test_validation_failure() {
        let mut processor = DataProcessor::new();
        let invalid_data = vec![150.0];
        
        let result = processor.process_dataset("test2", &invalid_data);
        assert!(result.is_err());
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
    pub tags: Vec<String>,
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
    config: HashMap<String, String>,
}

impl DataProcessor {
    pub fn new(config: HashMap<String, String>) -> Self {
        DataProcessor { config }
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if record.name.is_empty() {
            return Err(ProcessingError::ValidationError("Name cannot be empty".to_string()));
        }
        
        if record.value < 0.0 {
            return Err(ProcessingError::ValidationError("Value must be non-negative".to_string()));
        }
        
        if record.tags.len() > 10 {
            return Err(ProcessingError::ValidationError("Too many tags, maximum is 10".to_string()));
        }
        
        Ok(())
    }

    pub fn transform_record(&self, record: &DataRecord) -> Result<DataRecord, ProcessingError> {
        let mut transformed = record.clone();
        
        if let Some(prefix) = self.config.get("name_prefix") {
            transformed.name = format!("{}{}", prefix, transformed.name);
        }
        
        if let Some(factor_str) = self.config.get("value_multiplier") {
            if let Ok(factor) = factor_str.parse::<f64>() {
                transformed.value *= factor;
            } else {
                return Err(ProcessingError::TransformationFailed(
                    "Invalid multiplier in config".to_string()
                ));
            }
        }
        
        if let Some(default_tag) = self.config.get("default_tag") {
            if transformed.tags.is_empty() {
                transformed.tags.push(default_tag.clone());
            }
        }
        
        Ok(transformed)
    }

    pub fn process_batch(&self, records: Vec<DataRecord>) -> Result<Vec<DataRecord>, ProcessingError> {
        let mut processed = Vec::with_capacity(records.len());
        
        for record in records {
            self.validate_record(&record)?;
            let transformed = self.transform_record(&record)?;
            processed.push(transformed);
        }
        
        Ok(processed)
    }

    pub fn calculate_statistics(&self, records: &[DataRecord]) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        
        if records.is_empty() {
            return stats;
        }
        
        let count = records.len() as f64;
        let sum: f64 = records.iter().map(|r| r.value).sum();
        let avg = sum / count;
        
        let variance: f64 = records.iter()
            .map(|r| (r.value - avg).powi(2))
            .sum::<f64>() / count;
        
        stats.insert("count".to_string(), count);
        stats.insert("sum".to_string(), sum);
        stats.insert("average".to_string(), avg);
        stats.insert("variance".to_string(), variance);
        
        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_success() {
        let config = HashMap::new();
        let processor = DataProcessor::new(config);
        let record = DataRecord {
            id: 1,
            name: "Test".to_string(),
            value: 100.0,
            tags: vec!["tag1".to_string(), "tag2".to_string()],
        };
        
        assert!(processor.validate_record(&record).is_ok());
    }

    #[test]
    fn test_validation_failure() {
        let config = HashMap::new();
        let processor = DataProcessor::new(config);
        let record = DataRecord {
            id: 1,
            name: "".to_string(),
            value: -10.0,
            tags: vec!["tag".to_string(); 11],
        };
        
        assert!(processor.validate_record(&record).is_err());
    }

    #[test]
    fn test_transformation() {
        let mut config = HashMap::new();
        config.insert("name_prefix".to_string(), "PREFIX_".to_string());
        config.insert("value_multiplier".to_string(), "2.0".to_string());
        
        let processor = DataProcessor::new(config);
        let record = DataRecord {
            id: 1,
            name: "original".to_string(),
            value: 50.0,
            tags: Vec::new(),
        };
        
        let transformed = processor.transform_record(&record).unwrap();
        assert_eq!(transformed.name, "PREFIX_original");
        assert_eq!(transformed.value, 100.0);
    }
}