
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

        processor.register_validator("email", |s| s.contains('@') && s.contains('.'));
        processor.register_validator("numeric", |s| s.chars().all(|c| c.is_ascii_digit()));
        
        processor.register_transformer("uppercase", |s| s.to_uppercase());
        processor.register_transformer("trim", |s| s.trim().to_string());

        processor
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

    pub fn validate(&self, data: &str, validator_name: &str) -> bool {
        self.validators
            .get(validator_name)
            .map_or(false, |validator| validator(data))
    }

    pub fn transform(&self, data: String, transformer_name: &str) -> Option<String> {
        self.transformers
            .get(transformer_name)
            .map(|transformer| transformer(data))
    }

    pub fn process_pipeline(&self, data: String, operations: &[(&str, &str)]) -> Result<String, String> {
        let mut result = data;

        for (op_type, op_name) in operations {
            match *op_type {
                "validate" => {
                    if !self.validate(&result, op_name) {
                        return Err(format!("Validation failed for '{}'", op_name));
                    }
                }
                "transform" => {
                    result = self.transform(result, op_name)
                        .ok_or_else(|| format!("Unknown transformer '{}'", op_name))?;
                }
                _ => return Err(format!("Unknown operation type '{}'", op_type)),
            }
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_validation() {
        let processor = DataProcessor::new();
        assert!(processor.validate("test@example.com", "email"));
        assert!(!processor.validate("invalid-email", "email"));
    }

    #[test]
    fn test_numeric_validation() {
        let processor = DataProcessor::new();
        assert!(processor.validate("12345", "numeric"));
        assert!(!processor.validate("12a45", "numeric"));
    }

    #[test]
    fn test_uppercase_transformation() {
        let processor = DataProcessor::new();
        let result = processor.transform("hello".to_string(), "uppercase");
        assert_eq!(result, Some("HELLO".to_string()));
    }

    #[test]
    fn test_processing_pipeline() {
        let processor = DataProcessor::new();
        let operations = vec![
            ("validate", "email"),
            ("transform", "uppercase"),
        ];
        
        let result = processor.process_pipeline("test@example.com".to_string(), &operations);
        assert_eq!(result, Ok("TEST@EXAMPLE.COM".to_string()));
    }
}use std::collections::HashMap;

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

    pub fn process_dataset(&mut self, dataset_name: &str, data: &[f64]) -> Result<Vec<f64>, String> {
        if data.is_empty() {
            return Err("Dataset cannot be empty".to_string());
        }

        self.validate_data(data)?;
        
        let processed_data = self.transform_data(data);
        self.cache.insert(dataset_name.to_string(), processed_data.clone());
        
        Ok(processed_data)
    }

    fn validate_data(&self, data: &[f64]) -> Result<(), String> {
        for value in data {
            if value.is_nan() || value.is_infinite() {
                return Err("Invalid numeric value detected".to_string());
            }
        }
        Ok(())
    }

    fn transform_data(&self, data: &[f64]) -> Vec<f64> {
        let mean = self.calculate_mean(data);
        let std_dev = self.calculate_std_dev(data, mean);
        
        data.iter()
            .map(|&x| (x - mean) / std_dev)
            .collect()
    }

    fn calculate_mean(&self, data: &[f64]) -> f64 {
        let sum: f64 = data.iter().sum();
        sum / data.len() as f64
    }

    fn calculate_std_dev(&self, data: &[f64], mean: f64) -> f64 {
        let variance: f64 = data.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / data.len() as f64;
        
        variance.sqrt()
    }

    pub fn get_cached_data(&self, dataset_name: &str) -> Option<&Vec<f64>> {
        self.cache.get(dataset_name)
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
}

impl ValidationRule {
    pub fn new(field_name: &str, min_value: f64, max_value: f64, required: bool) -> Self {
        ValidationRule {
            field_name: field_name.to_string(),
            min_value,
            max_value,
            required,
        }
    }
}use csv::Reader;
use serde::Deserialize;
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

pub fn process_data_file(path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(path)?;
    let mut rdr = Reader::from_reader(file);
    let mut records = Vec::new();

    for result in rdr.deserialize() {
        let record: Record = result?;
        validate_record(&record)?;
        records.push(record);
    }

    Ok(records)
}

fn validate_record(record: &Record) -> Result<(), Box<dyn Error>> {
    if record.name.is_empty() {
        return Err("Name cannot be empty".into());
    }
    if record.value < 0.0 {
        return Err("Value cannot be negative".into());
    }
    if !["A", "B", "C"].contains(&record.category.as_str()) {
        return Err("Invalid category".into());
    }
    Ok(())
}

pub fn calculate_total(records: &[Record]) -> f64 {
    records.iter().map(|r| r.value).sum()
}

pub fn filter_by_category(records: Vec<Record>, category: &str) -> Vec<Record> {
    records
        .into_iter()
        .filter(|r| r.category == category)
        .collect()
}use std::collections::HashMap;

pub struct DataProcessor {
    cache: HashMap<String, Vec<f64>>,
    validation_rules: Vec<ValidationRule>,
}

#[derive(Debug, Clone)]
pub struct ValidationRule {
    pub field_name: String,
    pub min_value: f64,
    pub max_value: f64,
    pub required: bool,
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

    pub fn process_data(&mut self, dataset: &[HashMap<String, f64>]) -> Result<Vec<HashMap<String, f64>>, String> {
        let mut processed = Vec::new();

        for (index, record) in dataset.iter().enumerate() {
            match self.validate_record(record) {
                Ok(validated) => {
                    let transformed = self.transform_record(&validated);
                    self.cache_record(index, &transformed);
                    processed.push(transformed);
                }
                Err(e) => return Err(format!("Validation failed at record {}: {}", index, e)),
            }
        }

        Ok(processed)
    }

    fn validate_record(&self, record: &HashMap<String, f64>) -> Result<HashMap<String, f64>, String> {
        let mut validated = HashMap::new();

        for rule in &self.validation_rules {
            match record.get(&rule.field_name) {
                Some(&value) => {
                    if value < rule.min_value || value > rule.max_value {
                        return Err(format!("Field '{}' value {} out of range [{}, {}]", 
                            rule.field_name, value, rule.min_value, rule.max_value));
                    }
                    validated.insert(rule.field_name.clone(), value);
                }
                None => {
                    if rule.required {
                        return Err(format!("Required field '{}' missing", rule.field_name));
                    }
                }
            }
        }

        Ok(validated)
    }

    fn transform_record(&self, record: &HashMap<String, f64>) -> HashMap<String, f64> {
        let mut transformed = record.clone();
        
        for (key, value) in transformed.iter_mut() {
            if key.starts_with("normalized_") {
                *value = (*value * 100.0).round() / 100.0;
            }
        }

        transformed
    }

    fn cache_record(&mut self, index: usize, record: &HashMap<String, f64>) {
        let key = format!("record_{}", index);
        let values: Vec<f64> = record.values().cloned().collect();
        self.cache.insert(key, values);
    }

    pub fn get_cached_data(&self, index: usize) -> Option<&Vec<f64>> {
        let key = format!("record_{}", index);
        self.cache.get(&key)
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    pub fn calculate_statistics(&self) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        
        for values in self.cache.values() {
            if !values.is_empty() {
                let sum: f64 = values.iter().sum();
                let avg = sum / values.len() as f64;
                let max = values.iter().fold(f64::MIN, |a, &b| a.max(b));
                let min = values.iter().fold(f64::MAX, |a, &b| a.min(b));
                
                stats.insert("total_records".to_string(), self.cache.len() as f64);
                stats.insert("average_value".to_string(), avg);
                stats.insert("max_value".to_string(), max);
                stats.insert("min_value".to_string(), min);
            }
        }

        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_processing() {
        let mut processor = DataProcessor::new();
        
        processor.add_validation_rule(ValidationRule {
            field_name: "temperature".to_string(),
            min_value: -50.0,
            max_value: 100.0,
            required: true,
        });

        processor.add_validation_rule(ValidationRule {
            field_name: "humidity".to_string(),
            min_value: 0.0,
            max_value: 100.0,
            required: false,
        });

        let test_data = vec![
            [("temperature".to_string(), 25.5), ("humidity".to_string(), 60.0)]
                .iter().cloned().collect(),
            [("temperature".to_string(), 30.0), ("humidity".to_string(), 75.5)]
                .iter().cloned().collect(),
        ];

        let result = processor.process_data(&test_data);
        assert!(result.is_ok());
        
        let processed = result.unwrap();
        assert_eq!(processed.len(), 2);
        
        let stats = processor.calculate_statistics();
        assert!(stats.contains_key("total_records"));
    }

    #[test]
    fn test_validation_failure() {
        let mut processor = DataProcessor::new();
        
        processor.add_validation_rule(ValidationRule {
            field_name: "pressure".to_string(),
            min_value: 900.0,
            max_value: 1100.0,
            required: true,
        });

        let invalid_data = vec![
            [("pressure".to_string(), 850.0)].iter().cloned().collect(),
        ];

        let result = processor.process_data(&invalid_data);
        assert!(result.is_err());
    }
}