
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

    pub fn calculate_statistics(&self) -> HashMap<String, Stats> {
        let mut results = HashMap::new();
        
        for (key, values) in &self.data {
            if values.is_empty() {
                results.insert(key.clone(), Stats::default());
                continue;
            }

            let sum: f64 = values.iter().sum();
            let count = values.len() as f64;
            let mean = sum / count;
            
            let variance: f64 = values.iter()
                .map(|&x| (x - mean).powi(2))
                .sum::<f64>() / count;
            
            let std_dev = variance.sqrt();
            
            let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
            let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
            
            results.insert(key.clone(), Stats {
                mean,
                std_dev,
                min,
                max,
                count: values.len(),
            });
        }
        
        results
    }

    pub fn normalize_data(&mut self) {
        for values in self.data.values_mut() {
            if values.is_empty() {
                continue;
            }
            
            let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
            let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
            let range = max - min;
            
            if range > 0.0 {
                for value in values.iter_mut() {
                    *value = (*value - min) / range;
                }
            }
        }
    }

    pub fn get_data_keys(&self) -> Vec<String> {
        self.data.keys().cloned().collect()
    }

    pub fn has_data(&self) -> bool {
        !self.data.is_empty()
    }
}

#[derive(Debug, Clone)]
pub struct Stats {
    pub mean: f64,
    pub std_dev: f64,
    pub min: f64,
    pub max: f64,
    pub count: usize,
}

impl Default for Stats {
    fn default() -> Self {
        Stats {
            mean: 0.0,
            std_dev: 0.0,
            min: 0.0,
            max: 0.0,
            count: 0,
        }
    }
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
    fn test_data_processor_validation() {
        let rules = ValidationRules::new(
            0.0,
            100.0,
            vec!["temperature".to_string(), "humidity".to_string()]
        );
        
        let mut processor = DataProcessor::new(rules);
        
        assert!(processor.add_dataset("temperature".to_string(), vec![25.0, 30.0, 35.0]).is_ok());
        assert!(processor.add_dataset("pressure".to_string(), vec![1013.0]).is_err());
        assert!(processor.add_dataset("humidity".to_string(), vec![-5.0]).is_err());
    }

    #[test]
    fn test_statistics_calculation() {
        let rules = ValidationRules::new(
            f64::NEG_INFINITY,
            f64::INFINITY,
            vec!["test".to_string()]
        );
        
        let mut processor = DataProcessor::new(rules);
        processor.add_dataset("test".to_string(), vec![1.0, 2.0, 3.0, 4.0, 5.0]).unwrap();
        
        let stats = processor.calculate_statistics();
        let test_stats = stats.get("test").unwrap();
        
        assert_eq!(test_stats.mean, 3.0);
        assert_eq!(test_stats.min, 1.0);
        assert_eq!(test_stats.max, 5.0);
        assert_eq!(test_stats.count, 5);
    }

    #[test]
    fn test_data_normalization() {
        let rules = ValidationRules::new(
            f64::NEG_INFINITY,
            f64::INFINITY,
            vec!["values".to_string()]
        );
        
        let mut processor = DataProcessor::new(rules);
        processor.add_dataset("values".to_string(), vec![10.0, 20.0, 30.0, 40.0]).unwrap();
        
        processor.normalize_data();
        
        let data = processor.data.get("values").unwrap();
        assert_eq!(data[0], 0.0);
        assert_eq!(data[3], 1.0);
    }
}
use std::collections::HashMap;

pub struct DataProcessor {
    validation_rules: HashMap<String, ValidationRule>,
    transformation_pipeline: Vec<Transformation>,
}

pub struct ValidationRule {
    field_name: String,
    validator: Box<dyn Fn(&str) -> bool>,
    error_message: String,
}

pub enum Transformation {
    TrimWhitespace,
    Lowercase,
    Uppercase,
    ReplaceAll { pattern: String, replacement: String },
    Custom(Box<dyn Fn(String) -> String>),
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            validation_rules: HashMap::new(),
            transformation_pipeline: Vec::new(),
        }
    }

    pub fn add_validation_rule(&mut self, rule: ValidationRule) {
        self.validation_rules.insert(rule.field_name.clone(), rule);
    }

    pub fn add_transformation(&mut self, transformation: Transformation) {
        self.transformation_pipeline.push(transformation);
    }

    pub fn process_record(&self, record: &mut HashMap<String, String>) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        for (field_name, validation_rule) in &self.validation_rules {
            if let Some(value) = record.get(field_name) {
                if !(validation_rule.validator)(value) {
                    errors.push(format!("{}: {}", field_name, validation_rule.error_message));
                }
            }
        }

        if !errors.is_empty() {
            return Err(errors);
        }

        for transformation in &self.transformation_pipeline {
            self.apply_transformation(record, transformation);
        }

        Ok(())
    }

    fn apply_transformation(&self, record: &mut HashMap<String, String>, transformation: &Transformation) {
        match transformation {
            Transformation::TrimWhitespace => {
                for value in record.values_mut() {
                    *value = value.trim().to_string();
                }
            }
            Transformation::Lowercase => {
                for value in record.values_mut() {
                    *value = value.to_lowercase();
                }
            }
            Transformation::Uppercase => {
                for value in record.values_mut() {
                    *value = value.to_uppercase();
                }
            }
            Transformation::ReplaceAll { pattern, replacement } => {
                for value in record.values_mut() {
                    *value = value.replace(pattern, replacement);
                }
            }
            Transformation::Custom(func) => {
                for value in record.values_mut() {
                    *value = func(value.clone());
                }
            }
        }
    }
}

impl ValidationRule {
    pub fn new<F>(field_name: String, validator: F, error_message: String) -> Self
    where
        F: Fn(&str) -> bool + 'static,
    {
        ValidationRule {
            field_name,
            validator: Box::new(validator),
            error_message,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        
        processor.add_validation_rule(ValidationRule::new(
            "email".to_string(),
            |value| value.contains('@'),
            "Email must contain @ symbol".to_string(),
        ));

        processor.add_transformation(Transformation::TrimWhitespace);
        processor.add_transformation(Transformation::Lowercase);

        let mut test_record = HashMap::new();
        test_record.insert("email".to_string(), "  TEST@EXAMPLE.COM  ".to_string());
        test_record.insert("name".to_string(), "  John Doe  ".to_string());

        let result = processor.process_record(&mut test_record);
        assert!(result.is_ok());
        assert_eq!(test_record.get("email"), Some(&"test@example.com".to_string()));
        assert_eq!(test_record.get("name"), Some(&"john doe".to_string()));
    }

    #[test]
    fn test_validation_failure() {
        let mut processor = DataProcessor::new();
        
        processor.add_validation_rule(ValidationRule::new(
            "email".to_string(),
            |value| value.contains('@'),
            "Invalid email format".to_string(),
        ));

        let mut invalid_record = HashMap::new();
        invalid_record.insert("email".to_string(), "invalid-email".to_string());

        let result = processor.process_record(&mut invalid_record);
        assert!(result.is_err());
        if let Err(errors) = result {
            assert!(errors[0].contains("Invalid email format"));
        }
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub category: String,
    pub valid: bool,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: String) -> Self {
        let valid = value >= 0.0 && !category.is_empty();
        DataRecord {
            id,
            value,
            category,
            valid,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.valid
    }
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<usize, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut count = 0;

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            if line_num == 0 {
                continue;
            }

            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() != 3 {
                continue;
            }

            let id = match parts[0].parse::<u32>() {
                Ok(id) => id,
                Err(_) => continue,
            };

            let value = match parts[1].parse::<f64>() {
                Ok(value) => value,
                Err(_) => continue,
            };

            let category = parts[2].to_string();
            let record = DataRecord::new(id, value, category);
            self.records.push(record);
            count += 1;
        }

        Ok(count)
    }

    pub fn filter_valid(&self) -> Vec<&DataRecord> {
        self.records.iter().filter(|r| r.is_valid()).collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        let valid_records: Vec<&DataRecord> = self.filter_valid();
        if valid_records.is_empty() {
            return None;
        }

        let sum: f64 = valid_records.iter().map(|r| r.value).sum();
        Some(sum / valid_records.len() as f64)
    }

    pub fn group_by_category(&self) -> std::collections::HashMap<String, Vec<&DataRecord>> {
        let mut groups = std::collections::HashMap::new();
        
        for record in &self.records {
            if record.is_valid() {
                groups
                    .entry(record.category.clone())
                    .or_insert_with(Vec::new)
                    .push(record);
            }
        }
        
        groups
    }

    pub fn count_records(&self) -> usize {
        self.records.len()
    }

    pub fn count_valid(&self) -> usize {
        self.filter_valid().len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_record_validation() {
        let valid_record = DataRecord::new(1, 42.5, "A".to_string());
        assert!(valid_record.is_valid());

        let invalid_value = DataRecord::new(2, -5.0, "B".to_string());
        assert!(!invalid_value.is_valid());

        let invalid_category = DataRecord::new(3, 10.0, "".to_string());
        assert!(!invalid_category.is_valid());
    }

    #[test]
    fn test_data_processing() {
        let mut processor = DataProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category").unwrap();
        writeln!(temp_file, "1,10.5,TypeA").unwrap();
        writeln!(temp_file, "2,20.0,TypeB").unwrap();
        writeln!(temp_file, "3,-5.0,TypeA").unwrap();
        writeln!(temp_file, "4,15.0,").unwrap();
        
        let result = processor.load_from_csv(temp_file.path());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 4);
        assert_eq!(processor.count_records(), 4);
        assert_eq!(processor.count_valid(), 2);
        
        let average = processor.calculate_average();
        assert!(average.is_some());
        assert_eq!(average.unwrap(), 15.25);
        
        let groups = processor.group_by_category();
        assert_eq!(groups.len(), 2);
        assert_eq!(groups.get("TypeA").unwrap().len(), 1);
        assert_eq!(groups.get("TypeB").unwrap().len(), 1);
    }
}