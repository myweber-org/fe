
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
        if record.name.trim().is_empty() {
            return Err(ProcessingError::ValidationError(
                "Record name cannot be empty".to_string(),
            ));
        }

        if record.value < 0.0 {
            return Err(ProcessingError::ValidationError(
                "Record value must be non-negative".to_string(),
            ));
        }

        if let Some(max_tags) = self.config.get("max_tags") {
            if let Ok(max) = max_tags.parse::<usize>() {
                if record.tags.len() > max {
                    return Err(ProcessingError::ValidationError(format!(
                        "Record cannot have more than {} tags",
                        max
                    )));
                }
            }
        }

        Ok(())
    }

    pub fn transform_record(
        &self,
        record: &DataRecord,
    ) -> Result<DataRecord, ProcessingError> {
        let mut transformed = record.clone();

        if let Some(prefix) = self.config.get("name_prefix") {
            transformed.name = format!("{}{}", prefix, transformed.name);
        }

        if let Some(multiplier) = self.config.get("value_multiplier") {
            if let Ok(mult) = multiplier.parse::<f64>() {
                transformed.value *= mult;
            }
        }

        if let Some(default_tag) = self.config.get("default_tag") {
            if transformed.tags.is_empty() {
                transformed.tags.push(default_tag.clone());
            }
        }

        self.validate_record(&transformed)?;
        Ok(transformed)
    }

    pub fn process_batch(
        &self,
        records: Vec<DataRecord>,
    ) -> Result<Vec<DataRecord>, ProcessingError> {
        let mut processed = Vec::with_capacity(records.len());

        for (index, record) in records.into_iter().enumerate() {
            match self.transform_record(&record) {
                Ok(transformed) => processed.push(transformed),
                Err(e) => {
                    return Err(ProcessingError::TransformationFailed(format!(
                        "Failed to process record at index {}: {}",
                        index, e
                    )));
                }
            }
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
        let max = records
            .iter()
            .map(|r| r.value)
            .fold(f64::NEG_INFINITY, f64::max);
        let min = records
            .iter()
            .map(|r| r.value)
            .fold(f64::INFINITY, f64::min);

        stats.insert("count".to_string(), count);
        stats.insert("sum".to_string(), sum);
        stats.insert("average".to_string(), avg);
        stats.insert("maximum".to_string(), max);
        stats.insert("minimum".to_string(), min);

        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> HashMap<String, String> {
        let mut config = HashMap::new();
        config.insert("max_tags".to_string(), "3".to_string());
        config.insert("name_prefix".to_string(), "processed_".to_string());
        config.insert("value_multiplier".to_string(), "2.0".to_string());
        config.insert("default_tag".to_string(), "untagged".to_string());
        config
    }

    #[test]
    fn test_validate_record_valid() {
        let processor = DataProcessor::new(create_test_config());
        let record = DataRecord {
            id: 1,
            name: "test".to_string(),
            value: 10.0,
            tags: vec!["tag1".to_string(), "tag2".to_string()],
        };

        assert!(processor.validate_record(&record).is_ok());
    }

    #[test]
    fn test_validate_record_invalid_name() {
        let processor = DataProcessor::new(create_test_config());
        let record = DataRecord {
            id: 1,
            name: "".to_string(),
            value: 10.0,
            tags: vec![],
        };

        assert!(processor.validate_record(&record).is_err());
    }

    #[test]
    fn test_transform_record() {
        let processor = DataProcessor::new(create_test_config());
        let record = DataRecord {
            id: 1,
            name: "original".to_string(),
            value: 5.0,
            tags: vec![],
        };

        let transformed = processor.transform_record(&record).unwrap();
        assert_eq!(transformed.name, "processed_original");
        assert_eq!(transformed.value, 10.0);
        assert_eq!(transformed.tags, vec!["untagged"]);
    }

    #[test]
    fn test_process_batch() {
        let processor = DataProcessor::new(create_test_config());
        let records = vec![
            DataRecord {
                id: 1,
                name: "first".to_string(),
                value: 1.0,
                tags: vec!["a".to_string()],
            },
            DataRecord {
                id: 2,
                name: "second".to_string(),
                value: 2.0,
                tags: vec!["b".to_string(), "c".to_string()],
            },
        ];

        let processed = processor.process_batch(records).unwrap();
        assert_eq!(processed.len(), 2);
        assert_eq!(processed[0].value, 2.0);
        assert_eq!(processed[1].value, 4.0);
    }

    #[test]
    fn test_calculate_statistics() {
        let processor = DataProcessor::new(HashMap::new());
        let records = vec![
            DataRecord {
                id: 1,
                name: "a".to_string(),
                value: 10.0,
                tags: vec![],
            },
            DataRecord {
                id: 2,
                name: "b".to_string(),
                value: 20.0,
                tags: vec![],
            },
            DataRecord {
                id: 3,
                name: "c".to_string(),
                value: 30.0,
                tags: vec![],
            },
        ];

        let stats = processor.calculate_statistics(&records);
        assert_eq!(stats.get("count").unwrap(), &3.0);
        assert_eq!(stats.get("sum").unwrap(), &60.0);
        assert_eq!(stats.get("average").unwrap(), &20.0);
        assert_eq!(stats.get("maximum").unwrap(), &30.0);
        assert_eq!(stats.get("minimum").unwrap(), &10.0);
    }
}