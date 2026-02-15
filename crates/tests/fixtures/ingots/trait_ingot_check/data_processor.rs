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

        for rule in &self.validation_rules {
            if rule.required && data.is_empty() {
                return Err(format!("Field {} is required", rule.field_name));
            }

            for &value in data {
                if value < rule.min_value || value > rule.max_value {
                    return Err(format!(
                        "Value {} out of range for field {}",
                        value, rule.field_name
                    ));
                }
            }
        }

        Ok(())
    }

    fn transform_data(&self, data: &[f64]) -> Vec<f64> {
        let mean = data.iter().sum::<f64>() / data.len() as f64;
        let std_dev = self.calculate_std_dev(data, mean);

        data.iter()
            .map(|&x| (x - mean) / std_dev)
            .collect()
    }

    fn calculate_std_dev(&self, data: &[f64], mean: f64) -> f64 {
        let variance = data
            .iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>()
            / data.len() as f64;

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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_processing() {
        let mut processor = DataProcessor::new();
        let rule = ValidationRule::new("temperature", -50.0, 150.0, true);
        processor.add_validation_rule(rule);

        let data = vec![20.5, 22.3, 19.8, 21.2];
        let result = processor.process_dataset("test_data", &data);

        assert!(result.is_ok());
        assert_eq!(processor.get_cached_data("test_data").unwrap().len(), 4);
    }

    #[test]
    fn test_validation_failure() {
        let mut processor = DataProcessor::new();
        let rule = ValidationRule::new("pressure", 0.0, 100.0, true);
        processor.add_validation_rule(rule);

        let invalid_data = vec![150.0];
        let result = processor.process_dataset("invalid", &invalid_data);

        assert!(result.is_err());
    }
}