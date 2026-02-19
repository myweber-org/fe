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

    pub fn get_summary(&self) -> String {
        let count = self.data.len();
        let mean_str = match self.calculate_mean() {
            Some(m) => format!("{:.4}", m),
            None => "N/A".to_string(),
        };
        let std_dev_str = match self.calculate_standard_deviation() {
            Some(sd) => format!("{:.4}", sd),
            None => "N/A".to_string(),
        };

        format!(
            "Data Summary:\n  Count: {}\n  Mean: {}\n  Standard Deviation: {}",
            count, mean_str, std_dev_str
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_empty_data() {
        let processor = DataProcessor::new();
        assert_eq!(processor.calculate_mean(), None);
        assert_eq!(processor.calculate_standard_deviation(), None);
    }

    #[test]
    fn test_statistical_calculations() {
        let mut processor = DataProcessor::new();
        processor.data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        
        assert_eq!(processor.calculate_mean(), Some(3.0));
        assert!(processor.calculate_standard_deviation().unwrap() - 1.5811 < 0.0001);
    }

    #[test]
    fn test_csv_loading() -> Result<(), Box<dyn Error>> {
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "10.5\n20.3\n15.7\n")?;
        
        let mut processor = DataProcessor::new();
        processor.load_from_csv(temp_file.path().to_str().unwrap())?;
        
        assert_eq!(processor.data.len(), 3);
        assert_eq!(processor.data[0], 10.5);
        Ok(())
    }
}
use std::collections::HashMap;

pub struct DataProcessor {
    data: HashMap<String, Vec<f64>>,
    validation_rules: ValidationRules,
}

pub struct ValidationRules {
    min_value: f64,
    max_value: f64,
    required_keys: Vec<String>,
}

impl DataProcessor {
    pub fn new(rules: ValidationRules) -> Self {
        DataProcessor {
            data: HashMap::new(),
            validation_rules: rules,
        }
    }

    pub fn add_dataset(&mut self, key: String, values: Vec<f64>) -> Result<(), String> {
        if !self.validation_rules.required_keys.contains(&key) {
            return Err(format!("Key '{}' is not in required keys list", key));
        }

        for &value in &values {
            if value < self.validation_rules.min_value || value > self.validation_rules.max_value {
                return Err(format!("Value {} is outside allowed range [{}, {}]", 
                    value, self.validation_rules.min_value, self.validation_rules.max_value));
            }
        }

        self.data.insert(key, values);
        Ok(())
    }

    pub fn calculate_statistics(&self, key: &str) -> Option<Statistics> {
        self.data.get(key).map(|values| {
            let count = values.len();
            let sum: f64 = values.iter().sum();
            let mean = sum / count as f64;
            let variance: f64 = values.iter()
                .map(|&x| (x - mean).powi(2))
                .sum::<f64>() / count as f64;
            let std_dev = variance.sqrt();

            Statistics {
                count,
                sum,
                mean,
                variance,
                std_dev,
            }
        })
    }

    pub fn normalize_data(&mut self, key: &str) -> Result<(), String> {
        if let Some(values) = self.data.get_mut(key) {
            let stats = self.calculate_statistics(key).unwrap();
            
            for value in values {
                *value = (*value - stats.mean) / stats.std_dev;
            }
            Ok(())
        } else {
            Err(format!("Key '{}' not found in dataset", key))
        }
    }

    pub fn merge_datasets(&mut self, other: DataProcessor) {
        for (key, values) in other.data {
            self.data.entry(key)
                .and_modify(|existing| existing.extend_from_slice(&values))
                .or_insert(values);
        }
    }
}

pub struct Statistics {
    pub count: usize,
    pub sum: f64,
    pub mean: f64,
    pub variance: f64,
    pub std_dev: f64,
}

impl ValidationRules {
    pub fn new(min_value: f64, max_value: f64, required_keys: Vec<String>) -> Self {
        ValidationRules {
            min_value,
            max_value,
            required_keys,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_validation() {
        let rules = ValidationRules::new(
            0.0,
            100.0,
            vec!["temperature".to_string(), "humidity".to_string()]
        );
        let mut processor = DataProcessor::new(rules);

        assert!(processor.add_dataset("temperature".to_string(), vec![25.5, 30.0, 22.8]).is_ok());
        assert!(processor.add_dataset("pressure".to_string(), vec![1013.25]).is_err());
        assert!(processor.add_dataset("temperature".to_string(), vec![-5.0]).is_err());
    }

    #[test]
    fn test_statistics_calculation() {
        let rules = ValidationRules::new(f64::MIN, f64::MAX, vec!["test".to_string()]);
        let mut processor = DataProcessor::new(rules);
        
        processor.add_dataset("test".to_string(), vec![1.0, 2.0, 3.0, 4.0, 5.0]).unwrap();
        let stats = processor.calculate_statistics("test").unwrap();
        
        assert_eq!(stats.count, 5);
        assert_eq!(stats.sum, 15.0);
        assert_eq!(stats.mean, 3.0);
    }
}
use std::collections::HashMap;

pub struct DataProcessor {
    validators: HashMap<String, Box<dyn Fn(&str) -> bool>>,
    transformers: HashMap<String, Box<dyn Fn(String) -> String>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        let mut processor = DataProcessor {
            validators: HashMap::new(),
            transformers: HashMap::new(),
        };
        
        processor.register_default_validators();
        processor.register_default_transformers();
        
        processor
    }
    
    fn register_default_validators(&mut self) {
        self.validators.insert(
            "email".to_string(),
            Box::new(|s: &str| s.contains('@') && s.contains('.')),
        );
        
        self.validators.insert(
            "numeric".to_string(),
            Box::new(|s: &str| s.parse::<f64>().is_ok()),
        );
        
        self.validators.insert(
            "not_empty".to_string(),
            Box::new(|s: &str| !s.trim().is_empty()),
        );
    }
    
    fn register_default_transformers(&mut self) {
        self.transformers.insert(
            "uppercase".to_string(),
            Box::new(|s: String| s.to_uppercase()),
        );
        
        self.transformers.insert(
            "trim".to_string(),
            Box::new(|s: String| s.trim().to_string()),
        );
        
        self.transformers.insert(
            "reverse".to_string(),
            Box::new(|s: String| s.chars().rev().collect()),
        );
    }
    
    pub fn validate(&self, validator_name: &str, data: &str) -> bool {
        self.validators
            .get(validator_name)
            .map(|validator| validator(data))
            .unwrap_or(false)
    }
    
    pub fn transform(&self, transformer_name: &str, data: String) -> Option<String> {
        self.transformers
            .get(transformer_name)
            .map(|transformer| transformer(data))
    }
    
    pub fn process_pipeline(&self, data: &str, operations: Vec<(&str, &str)>) -> Result<String, String> {
        let mut result = data.to_string();
        
        for (op_type, op_name) in operations {
            match op_type {
                "validate" => {
                    if !self.validate(op_name, &result) {
                        return Err(format!("Validation '{}' failed for data: {}", op_name, result));
                    }
                }
                "transform" => {
                    result = self.transform(op_name, result)
                        .ok_or_else(|| format!("Unknown transformer: {}", op_name))?;
                }
                _ => return Err(format!("Unknown operation type: {}", op_type)),
            }
        }
        
        Ok(result)
    }
    
    pub fn register_validator<F>(&mut self, name: &str, validator: F)
    where
        F: Fn(&str) -> bool + 'static,
    {
        self.validators.insert(name.to_string(), Box::new(validator));
    }
    
    pub fn register_transformer<F>(&mut self, name: &str, transformer: F)
    where
        F: Fn(String) -> String + 'static,
    {
        self.transformers.insert(name.to_string(), Box::new(transformer));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_email_validation() {
        let processor = DataProcessor::new();
        assert!(processor.validate("email", "test@example.com"));
        assert!(!processor.validate("email", "invalid-email"));
    }
    
    #[test]
    fn test_numeric_validation() {
        let processor = DataProcessor::new();
        assert!(processor.validate("numeric", "123.45"));
        assert!(!processor.validate("numeric", "abc"));
    }
    
    #[test]
    fn test_transformation_pipeline() {
        let processor = DataProcessor::new();
        let operations = vec![
            ("validate", "not_empty"),
            ("transform", "uppercase"),
            ("transform", "reverse"),
        ];
        
        let result = processor.process_pipeline("hello", operations);
        assert_eq!(result, Ok("OLLEH".to_string()));
    }
    
    #[test]
    fn test_custom_validator() {
        let mut processor = DataProcessor::new();
        processor.register_validator("even_length", |s: &str| s.len() % 2 == 0);
        
        assert!(processor.validate("even_length", "abcd"));
        assert!(!processor.validate("even_length", "abc"));
    }
}use std::collections::HashMap;
use std::error::Error;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub timestamp: i64,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

impl DataRecord {
    pub fn new(id: u32, timestamp: i64, values: Vec<f64>) -> Self {
        Self {
            id,
            timestamp,
            values,
            metadata: HashMap::new(),
        }
    }

    pub fn add_metadata(&mut self, key: &str, value: &str) {
        self.metadata.insert(key.to_string(), value.to_string());
    }

    pub fn validate(&self) -> Result<(), Box<dyn Error>> {
        if self.id == 0 {
            return Err("Invalid record ID".into());
        }
        if self.timestamp < 0 {
            return Err("Invalid timestamp".into());
        }
        if self.values.is_empty() {
            return Err("Empty values array".into());
        }
        for &value in &self.values {
            if value.is_nan() || value.is_infinite() {
                return Err("Invalid numeric value".into());
            }
        }
        Ok(())
    }
}

pub fn process_records(records: &[DataRecord]) -> Vec<DataRecord> {
    records
        .iter()
        .filter(|record| record.validate().is_ok())
        .map(|record| {
            let mut processed = record.clone();
            let transformed_values: Vec<f64> = record
                .values
                .iter()
                .map(|&v| v * 2.0)
                .collect();
            processed.values = transformed_values;
            processed.add_metadata("processed", "true");
            processed
        })
        .collect()
}

pub fn calculate_statistics(records: &[DataRecord]) -> HashMap<String, f64> {
    let mut stats = HashMap::new();
    
    if records.is_empty() {
        return stats;
    }

    let all_values: Vec<f64> = records
        .iter()
        .flat_map(|r| r.values.clone())
        .collect();

    let count = all_values.len() as f64;
    let sum: f64 = all_values.iter().sum();
    let mean = sum / count;

    let variance: f64 = all_values
        .iter()
        .map(|&v| (v - mean).powi(2))
        .sum::<f64>() / count;

    stats.insert("count".to_string(), count);
    stats.insert("sum".to_string(), sum);
    stats.insert("mean".to_string(), mean);
    stats.insert("variance".to_string(), variance);

    stats
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_validation() {
        let valid_record = DataRecord::new(1, 1234567890, vec![1.0, 2.0, 3.0]);
        assert!(valid_record.validate().is_ok());

        let invalid_record = DataRecord::new(0, 1234567890, vec![1.0, 2.0]);
        assert!(invalid_record.validate().is_err());
    }

    #[test]
    fn test_process_records() {
        let records = vec![
            DataRecord::new(1, 1000, vec![1.0, 2.0]),
            DataRecord::new(2, 2000, vec![3.0, 4.0]),
        ];
        
        let processed = process_records(&records);
        assert_eq!(processed.len(), 2);
        assert_eq!(processed[0].values, vec![2.0, 4.0]);
        assert_eq!(processed[1].metadata.get("processed").unwrap(), "true");
    }

    #[test]
    fn test_calculate_statistics() {
        let records = vec![
            DataRecord::new(1, 1000, vec![1.0, 2.0]),
            DataRecord::new(2, 2000, vec![3.0, 4.0]),
        ];
        
        let stats = calculate_statistics(&records);
        assert_eq!(stats.get("count").unwrap(), &4.0);
        assert_eq!(stats.get("sum").unwrap(), &10.0);
        assert_eq!(stats.get("mean").unwrap(), &2.5);
    }
}