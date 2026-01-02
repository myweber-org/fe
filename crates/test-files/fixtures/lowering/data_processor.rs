
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

    pub fn process_dataset(&mut self, dataset_name: &str, data: Vec<f64>) -> Result<Vec<f64>, String> {
        if data.is_empty() {
            return Err("Dataset cannot be empty".to_string());
        }

        for rule in &self.validation_rules {
            if rule.required && data.iter().any(|&x| x.is_nan()) {
                return Err(format!("Field '{}' contains invalid values", rule.field_name));
            }
        }

        let processed_data: Vec<f64> = data
            .iter()
            .map(|&value| {
                let mut result = value;
                for rule in &self.validation_rules {
                    if value < rule.min_value || value > rule.max_value {
                        result = value.clamp(rule.min_value, rule.max_value);
                    }
                }
                result
            })
            .collect();

        self.cache.insert(dataset_name.to_string(), processed_data.clone());
        Ok(processed_data)
    }

    pub fn get_cached_data(&self, dataset_name: &str) -> Option<&Vec<f64>> {
        self.cache.get(dataset_name)
    }

    pub fn calculate_statistics(&self, dataset_name: &str) -> Option<Statistics> {
        self.cache.get(dataset_name).map(|data| {
            let sum: f64 = data.iter().sum();
            let count = data.len() as f64;
            let mean = sum / count;
            
            let variance: f64 = data.iter()
                .map(|&value| (value - mean).powi(2))
                .sum::<f64>() / count;
            
            Statistics {
                mean,
                variance,
                min: *data.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap(),
                max: *data.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap(),
                count: data.len(),
            }
        })
    }
}

pub struct Statistics {
    pub mean: f64,
    pub variance: f64,
    pub min: f64,
    pub max: f64,
    pub count: usize,
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
        let rule = ValidationRule::new("temperature", -50.0, 100.0, true);
        processor.add_validation_rule(rule);

        let data = vec![25.0, 30.0, 150.0, -60.0, 45.0];
        let result = processor.process_dataset("test_data", data);
        
        assert!(result.is_ok());
        let processed = result.unwrap();
        assert_eq!(processed.len(), 5);
        assert_eq!(processed[2], 100.0);
        assert_eq!(processed[3], -50.0);
    }

    #[test]
    fn test_statistics_calculation() {
        let mut processor = DataProcessor::new();
        let data = vec![10.0, 20.0, 30.0, 40.0, 50.0];
        processor.process_dataset("stats_test", data).unwrap();
        
        let stats = processor.calculate_statistics("stats_test").unwrap();
        assert_eq!(stats.mean, 30.0);
        assert_eq!(stats.min, 10.0);
        assert_eq!(stats.max, 50.0);
        assert_eq!(stats.count, 5);
    }
}