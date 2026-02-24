
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
        if key.is_empty() {
            return Err("Dataset key cannot be empty".to_string());
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

    pub fn calculate_statistics(&self, key: &str) -> Option<DatasetStats> {
        self.data.get(key).map(|values| {
            let count = values.len();
            let sum: f64 = values.iter().sum();
            let mean = sum / count as f64;
            
            let variance: f64 = values.iter()
                .map(|&x| (x - mean).powi(2))
                .sum::<f64>() / count as f64;
            
            DatasetStats {
                count,
                sum,
                mean,
                variance,
                min: *values.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap(),
                max: *values.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap(),
            }
        })
    }

    pub fn validate_all_datasets(&self) -> Vec<String> {
        let mut errors = Vec::new();
        
        for required_key in &self.validation_rules.required_keys {
            if !self.data.contains_key(required_key) {
                errors.push(format!("Missing required dataset: {}", required_key));
            }
        }

        for (key, values) in &self.data {
            if values.is_empty() {
                errors.push(format!("Dataset '{}' is empty", key));
            }
        }

        errors
    }

    pub fn transform_data<F>(&mut self, key: &str, transformer: F) -> Result<(), String>
    where
        F: Fn(f64) -> f64,
    {
        if let Some(values) = self.data.get_mut(key) {
            for value in values.iter_mut() {
                *value = transformer(*value);
            }
            Ok(())
        } else {
            Err(format!("Dataset '{}' not found", key))
        }
    }
}

pub struct DatasetStats {
    pub count: usize,
    pub sum: f64,
    pub mean: f64,
    pub variance: f64,
    pub min: f64,
    pub max: f64,
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