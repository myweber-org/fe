
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