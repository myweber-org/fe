use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct DataProcessor {
    data: Vec<f64>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor { data: Vec::new() }
    }

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            if let Ok(value) = line.trim().parse::<f64>() {
                self.data.push(value);
            }
        }

        Ok(())
    }

    pub fn calculate_mean(&self) -> Option<f64> {
        if self.data.is_empty() {
            return None;
        }

        let sum: f64 = self.data.iter().sum();
        Some(sum / self.data.len() as f64)
    }

    pub fn calculate_standard_deviation(&self) -> Option<f64> {
        if self.data.len() < 2 {
            return None;
        }

        let mean = self.calculate_mean()?;
        let variance: f64 = self.data
            .iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / (self.data.len() - 1) as f64;

        Some(variance.sqrt())
    }

    pub fn find_min_max(&self) -> Option<(f64, f64)> {
        if self.data.is_empty() {
            return None;
        }

        let min = self.data.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = self.data.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

        Some((min, max))
    }

    pub fn filter_outliers(&self, threshold: f64) -> Vec<f64> {
        if let Some(mean) = self.calculate_mean() {
            if let Some(std_dev) = self.calculate_standard_deviation() {
                return self.data
                    .iter()
                    .filter(|&&x| (x - mean).abs() <= threshold * std_dev)
                    .cloned()
                    .collect();
            }
        }
        self.data.clone()
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
        writeln!(temp_file, "10.5\n20.3\n15.7\n25.1\n18.9").unwrap();
        
        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        
        assert_eq!(processor.data.len(), 5);
        
        let mean = processor.calculate_mean().unwrap();
        assert!((mean - 18.1).abs() < 0.01);
        
        let (min, max) = processor.find_min_max().unwrap();
        assert_eq!(min, 10.5);
        assert_eq!(max, 25.1);
        
        let filtered = processor.filter_outliers(2.0);
        assert_eq!(filtered.len(), 5);
    }
}
use std::collections::HashMap;

pub struct DataProcessor {
    cache: HashMap<String, Vec<f64>>,
    validation_rules: Vec<ValidationRule>,
}

pub struct ValidationRule {
    field_name: String,
    min_value: f64,
    max_value: f64,
    required: bool,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            cache: HashMap::new(),
            validation_rules: Vec::new(),
        }
    }

    pub fn add_validation_rule(&mut self, rule: ValidationRule) {
        self.validation_rules.push(rule);
    }

    pub fn process_dataset(&mut self, dataset: &[HashMap<String, f64>]) -> Result<Vec<ProcessedRecord>, ProcessingError> {
        let mut results = Vec::with_capacity(dataset.len());
        
        for (index, record) in dataset.iter().enumerate() {
            match self.validate_record(record) {
                Ok(validated) => {
                    let transformed = self.transform_record(&validated);
                    self.cache_results(&transformed);
                    results.push(transformed);
                }
                Err(err) => {
                    return Err(ProcessingError::ValidationFailed {
                        record_index: index,
                        reason: err,
                    });
                }
            }
        }
        
        Ok(results)
    }

    fn validate_record(&self, record: &HashMap<String, f64>) -> Result<ValidatedRecord, String> {
        for rule in &self.validation_rules {
            if let Some(&value) = record.get(&rule.field_name) {
                if value < rule.min_value || value > rule.max_value {
                    return Err(format!(
                        "Field '{}' value {} out of range [{}, {}]",
                        rule.field_name, value, rule.min_value, rule.max_value
                    ));
                }
            } else if rule.required {
                return Err(format!("Required field '{}' missing", rule.field_name));
            }
        }
        
        Ok(ValidatedRecord {
            original: record.clone(),
            validation_timestamp: std::time::SystemTime::now(),
        })
    }

    fn transform_record(&self, validated: &ValidatedRecord) -> ProcessedRecord {
        let mut transformed = HashMap::new();
        
        for (key, value) in &validated.original {
            let transformed_value = match key.as_str() {
                "temperature" => (value - 32.0) * 5.0 / 9.0,
                "pressure" => value * 0.0689476,
                "humidity" => value.min(100.0).max(0.0),
                _ => *value,
            };
            transformed.insert(key.clone(), transformed_value);
        }
        
        ProcessedRecord {
            data: transformed,
            processing_timestamp: std::time::SystemTime::now(),
            validation_timestamp: validated.validation_timestamp,
        }
    }

    fn cache_results(&mut self, processed: &ProcessedRecord) {
        for (key, value) in &processed.data {
            self.cache
                .entry(key.clone())
                .or_insert_with(Vec::new)
                .push(*value);
        }
    }

    pub fn get_cached_stats(&self, field: &str) -> Option<FieldStatistics> {
        self.cache.get(field).map(|values| {
            let count = values.len();
            let sum: f64 = values.iter().sum();
            let mean = sum / count as f64;
            let variance: f64 = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / count as f64;
            
            FieldStatistics {
                field_name: field.to_string(),
                count,
                mean,
                variance,
                min: *values.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap_or(&0.0),
                max: *values.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap_or(&0.0),
            }
        })
    }
}

pub struct ValidatedRecord {
    original: HashMap<String, f64>,
    validation_timestamp: std::time::SystemTime,
}

pub struct ProcessedRecord {
    data: HashMap<String, f64>,
    processing_timestamp: std::time::SystemTime,
    validation_timestamp: std::time::SystemTime,
}

pub struct FieldStatistics {
    pub field_name: String,
    pub count: usize,
    pub mean: f64,
    pub variance: f64,
    pub min: f64,
    pub max: f64,
}

#[derive(Debug)]
pub enum ProcessingError {
    ValidationFailed {
        record_index: usize,
        reason: String,
    },
    TransformationError(String),
    CacheError(String),
}

impl std::fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProcessingError::ValidationFailed { record_index, reason } => {
                write!(f, "Validation failed at record {}: {}", record_index, reason)
            }
            ProcessingError::TransformationError(msg) => {
                write!(f, "Transformation error: {}", msg)
            }
            ProcessingError::CacheError(msg) => {
                write!(f, "Cache error: {}", msg)
            }
        }
    }
}

impl std::error::Error for ProcessingError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_processing() {
        let mut processor = DataProcessor::new();
        
        processor.add_validation_rule(ValidationRule {
            field_name: "temperature".to_string(),
            min_value: -50.0,
            max_value: 150.0,
            required: true,
        });
        
        let dataset = vec![
            [("temperature".to_string(), 68.0), ("humidity".to_string(), 45.0)]
                .iter().cloned().collect(),
            [("temperature".to_string(), 72.0), ("pressure".to_string(), 14.7)]
                .iter().cloned().collect(),
        ];
        
        let result = processor.process_dataset(&dataset);
        assert!(result.is_ok());
        
        let stats = processor.get_cached_stats("temperature");
        assert!(stats.is_some());
        
        if let Some(stats) = stats {
            assert_eq!(stats.count, 2);
            assert!(stats.mean > 0.0);
        }
    }
}