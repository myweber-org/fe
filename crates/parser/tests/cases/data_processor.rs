
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub category: String,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: String) -> Self {
        DataRecord { id, value, category }
    }

    pub fn is_valid(&self) -> bool {
        self.id > 0 && self.value.is_finite() && !self.category.is_empty()
    }
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor { records: Vec::new() }
    }

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<usize, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut count = 0;

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            if line_num == 0 || line.trim().is_empty() {
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

            let category = parts[2].trim().to_string();

            let record = DataRecord::new(id, value, category);
            if record.is_valid() {
                self.records.push(record);
                count += 1;
            }
        }

        Ok(count)
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }

        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn get_statistics(&self) -> (f64, f64, f64) {
        if self.records.is_empty() {
            return (0.0, 0.0, 0.0);
        }

        let values: Vec<f64> = self.records.iter().map(|r| r.value).collect();
        let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let avg = self.calculate_average().unwrap_or(0.0);

        (min, max, avg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_record_validation() {
        let valid_record = DataRecord::new(1, 42.5, "type_a".to_string());
        assert!(valid_record.is_valid());

        let invalid_id = DataRecord::new(0, 42.5, "type_a".to_string());
        assert!(!invalid_id.is_valid());

        let invalid_value = DataRecord::new(1, f64::NAN, "type_a".to_string());
        assert!(!invalid_value.is_valid());

        let invalid_category = DataRecord::new(1, 42.5, "".to_string());
        assert!(!invalid_category.is_valid());
    }

    #[test]
    fn test_csv_loading() {
        let mut csv_content = "id,value,category\n".to_string();
        csv_content.push_str("1,10.5,alpha\n");
        csv_content.push_str("2,20.3,beta\n");
        csv_content.push_str("3,15.7,alpha\n");

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", csv_content).unwrap();

        let mut processor = DataProcessor::new();
        let result = processor.load_from_csv(temp_file.path());

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 3);
        assert_eq!(processor.records.len(), 3);
    }

    #[test]
    fn test_filter_and_statistics() {
        let mut processor = DataProcessor::new();
        processor.records.push(DataRecord::new(1, 10.0, "A".to_string()));
        processor.records.push(DataRecord::new(2, 20.0, "B".to_string()));
        processor.records.push(DataRecord::new(3, 30.0, "A".to_string()));

        let filtered = processor.filter_by_category("A");
        assert_eq!(filtered.len(), 2);

        let stats = processor.get_statistics();
        assert_eq!(stats.0, 10.0);
        assert_eq!(stats.1, 30.0);
        assert_eq!(stats.2, 20.0);
    }
}
use std::collections::HashMap;

pub struct DataProcessor {
    validators: HashMap<String, Box<dyn Fn(&str) -> bool>>,
    transformers: HashMap<String, Box<dyn Fn(String) -> String>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            validators: HashMap::new(),
            transformers: HashMap::new(),
        }
    }

    pub fn add_validator(&mut self, name: &str, validator: Box<dyn Fn(&str) -> bool>) {
        self.validators.insert(name.to_string(), validator);
    }

    pub fn add_transformer(&mut self, name: &str, transformer: Box<dyn Fn(String) -> String>) {
        self.transformers.insert(name.to_string(), transformer);
    }

    pub fn process_data(&self, data: &str, validator_name: &str, transformer_name: &str) -> Option<String> {
        let validator = self.validators.get(validator_name)?;
        
        if !validator(data) {
            return None;
        }

        let transformer = self.transformers.get(transformer_name)?;
        Some(transformer(data.to_string()))
    }

    pub fn validate_email(&self, email: &str) -> bool {
        email.contains('@') && email.contains('.')
    }

    pub fn uppercase_transform(&self, input: String) -> String {
        input.to_uppercase()
    }
}

pub fn initialize_default_processor() -> DataProcessor {
    let mut processor = DataProcessor::new();
    
    processor.add_validator("email", Box::new(|s| s.contains('@') && s.contains('.')));
    processor.add_validator("numeric", Box::new(|s| s.chars().all(|c| c.is_numeric())));
    
    processor.add_transformer("uppercase", Box::new(|s| s.to_uppercase()));
    processor.add_transformer("reverse", Box::new(|s| s.chars().rev().collect()));
    
    processor
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_validation() {
        let processor = DataProcessor::new();
        assert!(processor.validate_email("test@example.com"));
        assert!(!processor.validate_email("invalid-email"));
    }

    #[test]
    fn test_uppercase_transform() {
        let processor = DataProcessor::new();
        assert_eq!(processor.uppercase_transform("hello".to_string()), "HELLO");
    }

    #[test]
    fn test_process_data_flow() {
        let processor = initialize_default_processor();
        
        let result = processor.process_data("test@example.com", "email", "uppercase");
        assert_eq!(result, Some("TEST@EXAMPLE.COM".to_string()));
        
        let invalid_result = processor.process_data("invalid", "email", "uppercase");
        assert_eq!(invalid_result, None);
    }
}
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug)]
pub enum DataError {
    InvalidId,
    EmptyValues,
    InvalidValue(f64),
    MissingMetadata(String),
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataError::InvalidId => write!(f, "Record ID must be greater than zero"),
            DataError::EmptyValues => write!(f, "Record must contain at least one value"),
            DataError::InvalidValue(val) => write!(f, "Invalid value detected: {}", val),
            DataError::MissingMetadata(key) => write!(f, "Required metadata missing: {}", key),
        }
    }
}

impl Error for DataError {}

pub struct DataProcessor {
    records: Vec<DataRecord>,
    validation_rules: HashMap<String, Box<dyn Fn(&DataRecord) -> Result<(), DataError>>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
            validation_rules: HashMap::new(),
        }
    }

    pub fn add_record(&mut self, record: DataRecord) -> Result<(), DataError> {
        self.validate_record(&record)?;
        self.records.push(record);
        Ok(())
    }

    pub fn add_validation_rule<F>(&mut self, name: &str, rule: F)
    where
        F: Fn(&DataRecord) -> Result<(), DataError> + 'static,
    {
        self.validation_rules.insert(name.to_string(), Box::new(rule));
    }

    pub fn process_records(&self) -> Vec<DataRecord> {
        self.records
            .iter()
            .filter_map(|record| self.transform_record(record).ok())
            .collect()
    }

    fn validate_record(&self, record: &DataRecord) -> Result<(), DataError> {
        if record.id == 0 {
            return Err(DataError::InvalidId);
        }

        if record.values.is_empty() {
            return Err(DataError::EmptyValues);
        }

        for &value in &record.values {
            if value.is_nan() || value.is_infinite() {
                return Err(DataError::InvalidValue(value));
            }
        }

        for (name, rule) in &self.validation_rules {
            if let Err(e) = rule(record) {
                return Err(e);
            }
        }

        Ok(())
    }

    fn transform_record(&self, record: &DataRecord) -> Result<DataRecord, DataError> {
        let transformed_values: Vec<f64> = record
            .values
            .iter()
            .map(|&v| v * 2.0)
            .collect();

        let mut transformed_metadata = record.metadata.clone();
        transformed_metadata.insert(
            "processed_timestamp".to_string(),
            chrono::Utc::now().to_rfc3339(),
        );

        Ok(DataRecord {
            id: record.id,
            values: transformed_values,
            metadata: transformed_metadata,
        })
    }

    pub fn calculate_statistics(&self) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        
        if self.records.is_empty() {
            return stats;
        }

        let all_values: Vec<f64> = self.records
            .iter()
            .flat_map(|r| r.values.clone())
            .collect();

        let sum: f64 = all_values.iter().sum();
        let count = all_values.len() as f64;
        let mean = sum / count;

        let variance: f64 = all_values
            .iter()
            .map(|&v| (v - mean).powi(2))
            .sum::<f64>() / count;

        stats.insert("mean".to_string(), mean);
        stats.insert("variance".to_string(), variance);
        stats.insert("total_records".to_string(), self.records.len() as f64);
        stats.insert("total_values".to_string(), count);

        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_record_processing() {
        let mut processor = DataProcessor::new();
        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), "test".to_string());

        let record = DataRecord {
            id: 1,
            values: vec![1.0, 2.0, 3.0],
            metadata,
        };

        assert!(processor.add_record(record).is_ok());
        assert_eq!(processor.records.len(), 1);
    }

    #[test]
    fn test_invalid_record_rejection() {
        let mut processor = DataProcessor::new();
        let record = DataRecord {
            id: 0,
            values: vec![],
            metadata: HashMap::new(),
        };

        assert!(processor.add_record(record).is_err());
    }

    #[test]
    fn test_custom_validation_rule() {
        let mut processor = DataProcessor::new();
        
        processor.add_validation_rule("max_values", |record| {
            if record.values.len() > 5 {
                Err(DataError::InvalidValue(record.values.len() as f64))
            } else {
                Ok(())
            }
        });

        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), "test".to_string());

        let valid_record = DataRecord {
            id: 1,
            values: vec![1.0, 2.0, 3.0],
            metadata: metadata.clone(),
        };

        let invalid_record = DataRecord {
            id: 2,
            values: vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0],
            metadata,
        };

        assert!(processor.add_record(valid_record).is_ok());
        assert!(processor.add_record(invalid_record).is_err());
    }
}