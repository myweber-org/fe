use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct DataProcessor {
    delimiter: char,
    has_header: bool,
}

impl DataProcessor {
    pub fn new(delimiter: char, has_header: bool) -> Self {
        DataProcessor {
            delimiter,
            has_header,
        }
    }

    pub fn process_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();
        let mut lines = reader.lines();

        if self.has_header {
            let _ = lines.next();
        }

        for line_result in lines {
            let line = line_result?;
            let fields: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();
            
            if !fields.is_empty() {
                records.push(fields);
            }
        }

        Ok(records)
    }

    pub fn validate_record(&self, record: &[String], expected_fields: usize) -> bool {
        record.len() == expected_fields && record.iter().all(|field| !field.is_empty())
    }

    pub fn extract_column(&self, data: &[Vec<String>], column_index: usize) -> Vec<String> {
        data.iter()
            .filter_map(|record| record.get(column_index).cloned())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_process_file_with_header() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,30,New York").unwrap();
        writeln!(temp_file, "Bob,25,London").unwrap();

        let processor = DataProcessor::new(',', true);
        let result = processor.process_file(temp_file.path()).unwrap();
        
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["Alice", "30", "New York"]);
    }

    #[test]
    fn test_validate_record() {
        let processor = DataProcessor::new(',', false);
        let valid_record = vec!["field1".to_string(), "field2".to_string()];
        let invalid_record = vec!["".to_string(), "field2".to_string()];
        
        assert!(processor.validate_record(&valid_record, 2));
        assert!(!processor.validate_record(&invalid_record, 2));
    }

    #[test]
    fn test_extract_column() {
        let data = vec![
            vec!["a".to_string(), "b".to_string()],
            vec!["c".to_string(), "d".to_string()],
        ];
        let processor = DataProcessor::new(',', false);
        let column = processor.extract_column(&data, 1);
        
        assert_eq!(column, vec!["b".to_string(), "d".to_string()]);
    }
}
use std::collections::HashMap;

pub struct DataProcessor {
    data: HashMap<String, Vec<f64>>,
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
            data: HashMap::new(),
            validation_rules: Vec::new(),
        }
    }

    pub fn add_dataset(&mut self, name: String, values: Vec<f64>) {
        self.data.insert(name, values);
    }

    pub fn add_validation_rule(&mut self, rule: ValidationRule) {
        self.validation_rules.push(rule);
    }

    pub fn validate_all(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        for rule in &self.validation_rules {
            if let Some(data) = self.data.get(&rule.field_name) {
                if rule.required && data.is_empty() {
                    errors.push(format!("Field '{}' is required but empty", rule.field_name));
                    continue;
                }

                for &value in data {
                    if value < rule.min_value || value > rule.max_value {
                        errors.push(format!(
                            "Value {} in field '{}' is outside valid range [{}, {}]",
                            value, rule.field_name, rule.min_value, rule.max_value
                        ));
                    }
                }
            } else if rule.required {
                errors.push(format!("Required field '{}' not found", rule.field_name));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    pub fn normalize_data(&mut self, field_name: &str) -> Option<Vec<f64>> {
        if let Some(data) = self.data.get_mut(field_name) {
            if data.is_empty() {
                return None;
            }

            let min = data.iter().fold(f64::INFINITY, |a, &b| a.min(b));
            let max = data.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
            
            if (max - min).abs() < f64::EPSILON {
                return Some(vec![0.0; data.len()]);
            }

            let normalized: Vec<f64> = data
                .iter()
                .map(|&x| (x - min) / (max - min))
                .collect();

            self.data.insert(field_name.to_string(), normalized.clone());
            Some(normalized)
        } else {
            None
        }
    }

    pub fn calculate_statistics(&self, field_name: &str) -> Option<Statistics> {
        if let Some(data) = self.data.get(field_name) {
            if data.is_empty() {
                return None;
            }

            let sum: f64 = data.iter().sum();
            let count = data.len() as f64;
            let mean = sum / count;

            let variance: f64 = data
                .iter()
                .map(|&x| (x - mean).powi(2))
                .sum::<f64>() / count;

            let sorted_data = {
                let mut sorted = data.clone();
                sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
                sorted
            };

            let median = if count as usize % 2 == 0 {
                let mid = count as usize / 2;
                (sorted_data[mid - 1] + sorted_data[mid]) / 2.0
            } else {
                sorted_data[count as usize / 2]
            };

            Some(Statistics {
                mean,
                median,
                variance,
                std_dev: variance.sqrt(),
                min: *sorted_data.first().unwrap(),
                max: *sorted_data.last().unwrap(),
                count: count as usize,
            })
        } else {
            None
        }
    }
}

pub struct Statistics {
    pub mean: f64,
    pub median: f64,
    pub variance: f64,
    pub std_dev: f64,
    pub min: f64,
    pub max: f64,
    pub count: usize,
}

impl ValidationRule {
    pub fn new(field_name: String, min_value: f64, max_value: f64, required: bool) -> Self {
        ValidationRule {
            field_name,
            min_value,
            max_value,
            required,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_validation() {
        let mut processor = DataProcessor::new();
        processor.add_dataset("temperature".to_string(), vec![20.5, 22.3, 18.7, 25.1]);
        
        let rule = ValidationRule::new("temperature".to_string(), 15.0, 30.0, true);
        processor.add_validation_rule(rule);

        assert!(processor.validate_all().is_ok());
    }

    #[test]
    fn test_normalization() {
        let mut processor = DataProcessor::new();
        processor.add_dataset("scores".to_string(), vec![10.0, 20.0, 30.0, 40.0]);
        
        let normalized = processor.normalize_data("scores").unwrap();
        assert_eq!(normalized, vec![0.0, 1.0/3.0, 2.0/3.0, 1.0]);
    }

    #[test]
    fn test_statistics_calculation() {
        let mut processor = DataProcessor::new();
        processor.add_dataset("values".to_string(), vec![1.0, 2.0, 3.0, 4.0, 5.0]);
        
        let stats = processor.calculate_statistics("values").unwrap();
        assert_eq!(stats.mean, 3.0);
        assert_eq!(stats.median, 3.0);
        assert_eq!(stats.variance, 2.0);
        assert_eq!(stats.std_dev, 2.0_f64.sqrt());
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct DataProcessor {
    delimiter: char,
    has_header: bool,
}

impl DataProcessor {
    pub fn new(delimiter: char, has_header: bool) -> Self {
        DataProcessor {
            delimiter,
            has_header,
        }
    }

    pub fn process_file<P: AsRef<Path>>(&self, file_path: P) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();

        for (line_number, line) in reader.lines().enumerate() {
            let line = line?;
            
            if line.trim().is_empty() {
                continue;
            }

            if self.has_header && line_number == 0 {
                continue;
            }

            let fields: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            if !self.validate_record(&fields) {
                return Err(format!("Invalid record at line {}", line_number + 1).into());
            }

            records.push(fields);
        }

        Ok(records)
    }

    fn validate_record(&self, fields: &[String]) -> bool {
        !fields.is_empty() && fields.iter().all(|field| !field.is_empty())
    }

    pub fn calculate_statistics(&self, records: &[Vec<String>], column_index: usize) -> Option<(f64, f64, f64)> {
        if records.is_empty() || column_index >= records[0].len() {
            return None;
        }

        let values: Vec<f64> = records
            .iter()
            .filter_map(|record| record[column_index].parse::<f64>().ok())
            .collect();

        if values.is_empty() {
            return None;
        }

        let sum: f64 = values.iter().sum();
        let count = values.len() as f64;
        let mean = sum / count;

        let variance: f64 = values
            .iter()
            .map(|value| (value - mean).powi(2))
            .sum::<f64>() / count;

        let std_dev = variance.sqrt();

        Some((mean, variance, std_dev))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_process_file_with_header() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,score").unwrap();
        writeln!(temp_file, "Alice,25,95.5").unwrap();
        writeln!(temp_file, "Bob,30,87.2").unwrap();
        writeln!(temp_file, "Charlie,28,91.8").unwrap();

        let processor = DataProcessor::new(',', true);
        let result = processor.process_file(temp_file.path()).unwrap();

        assert_eq!(result.len(), 3);
        assert_eq!(result[0], vec!["Alice", "25", "95.5"]);
    }

    #[test]
    fn test_process_file_without_header() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "Alice,25,95.5").unwrap();
        writeln!(temp_file, "Bob,30,87.2").unwrap();

        let processor = DataProcessor::new(',', false);
        let result = processor.process_file(temp_file.path()).unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["Alice", "25", "95.5"]);
    }

    #[test]
    fn test_calculate_statistics() {
        let records = vec![
            vec!["10.5".to_string(), "20.0".to_string()],
            vec!["15.5".to_string(), "25.0".to_string()],
            vec!["12.0".to_string(), "30.0".to_string()],
        ];

        let processor = DataProcessor::new(',', false);
        let stats = processor.calculate_statistics(&records, 0).unwrap();

        assert!((stats.0 - 12.666666666666666).abs() < 0.0001);
        assert!((stats.1 - 4.222222222222222).abs() < 0.0001);
        assert!((stats.2 - 2.054804667).abs() < 0.0001);
    }

    #[test]
    fn test_validate_record() {
        let processor = DataProcessor::new(',', false);
        
        let valid_record = vec!["field1".to_string(), "field2".to_string()];
        assert!(processor.validate_record(&valid_record));

        let empty_record = vec![];
        assert!(!processor.validate_record(&empty_record));

        let record_with_empty_field = vec!["field1".to_string(), "".to_string()];
        assert!(!processor.validate_record(&record_with_empty_field));
    }
}
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DataRecord {
    pub id: u64,
    pub timestamp: i64,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Error)]
pub enum ProcessingError {
    #[error("Invalid data format")]
    InvalidFormat,
    #[error("Data validation failed: {0}")]
    ValidationFailed(String),
    #[error("Transformation error: {0}")]
    TransformationError(String),
}

pub struct DataProcessor {
    validation_rules: Vec<ValidationRule>,
    transformation_pipeline: Vec<Transformation>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            validation_rules: Vec::new(),
            transformation_pipeline: Vec::new(),
        }
    }

    pub fn add_validation_rule(&mut self, rule: ValidationRule) {
        self.validation_rules.push(rule);
    }

    pub fn add_transformation(&mut self, transformation: Transformation) {
        self.transformation_pipeline.push(transformation);
    }

    pub fn process(&self, record: &mut DataRecord) -> Result<(), ProcessingError> {
        for rule in &self.validation_rules {
            rule.validate(record)?;
        }

        for transformation in &self.transformation_pipeline {
            transformation.apply(record)?;
        }

        Ok(())
    }

    pub fn batch_process(&self, records: &mut [DataRecord]) -> Vec<Result<(), ProcessingError>> {
        records
            .iter_mut()
            .map(|record| self.process(record))
            .collect()
    }
}

pub trait ValidationRule {
    fn validate(&self, record: &DataRecord) -> Result<(), ProcessingError>;
}

pub trait Transformation {
    fn apply(&self, record: &mut DataRecord) -> Result<(), ProcessingError>;
}

pub struct TimestampValidator {
    min_timestamp: i64,
    max_timestamp: i64,
}

impl TimestampValidator {
    pub fn new(min: i64, max: i64) -> Self {
        TimestampValidator {
            min_timestamp: min,
            max_timestamp: max,
        }
    }
}

impl ValidationRule for TimestampValidator {
    fn validate(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if record.timestamp < self.min_timestamp || record.timestamp > self.max_timestamp {
            Err(ProcessingError::ValidationFailed(format!(
                "Timestamp {} out of range [{}, {}]",
                record.timestamp, self.min_timestamp, self.max_timestamp
            )))
        } else {
            Ok(())
        }
    }
}

pub struct NormalizationTransformation {
    scale_factor: f64,
}

impl NormalizationTransformation {
    pub fn new(scale_factor: f64) -> Self {
        NormalizationTransformation { scale_factor }
    }
}

impl Transformation for NormalizationTransformation {
    fn apply(&self, record: &mut DataRecord) -> Result<(), ProcessingError> {
        if self.scale_factor == 0.0 {
            return Err(ProcessingError::TransformationError(
                "Scale factor cannot be zero".to_string(),
            ));
        }

        for value in record.values.iter_mut() {
            *value /= self.scale_factor;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_processing_pipeline() {
        let mut processor = DataProcessor::new();
        processor.add_validation_rule(Box::new(TimestampValidator::new(0, 1000)));
        processor.add_transformation(Box::new(NormalizationTransformation::new(10.0)));

        let mut record = DataRecord {
            id: 1,
            timestamp: 500,
            values: vec![20.0, 30.0, 40.0],
            metadata: HashMap::new(),
        };

        let result = processor.process(&mut record);
        assert!(result.is_ok());
        assert_eq!(record.values, vec![2.0, 3.0, 4.0]);
    }

    #[test]
    fn test_validation_failure() {
        let mut processor = DataProcessor::new();
        processor.add_validation_rule(Box::new(TimestampValidator::new(0, 100)));

        let mut record = DataRecord {
            id: 1,
            timestamp: 200,
            values: vec![1.0, 2.0, 3.0],
            metadata: HashMap::new(),
        };

        let result = processor.process(&mut record);
        assert!(result.is_err());
    }
}