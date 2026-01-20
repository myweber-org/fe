
use std::collections::HashMap;

pub struct DataProcessor {
    cache: HashMap<String, Vec<f64>>,
    validation_rules: Vec<ValidationRule>,
}

#[derive(Debug, Clone)]
pub struct ValidationRule {
    pub min_value: f64,
    pub max_value: f64,
    pub required: bool,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            cache: HashMap::new(),
            validation_rules: vec![
                ValidationRule {
                    min_value: 0.0,
                    max_value: 100.0,
                    required: true,
                },
            ],
        }
    }

    pub fn process_dataset(&mut self, dataset_id: &str, data: &[f64]) -> Result<Vec<f64>, String> {
        if data.is_empty() {
            return Err("Dataset cannot be empty".to_string());
        }

        for value in data {
            for rule in &self.validation_rules {
                if rule.required && (*value < rule.min_value || *value > rule.max_value) {
                    return Err(format!(
                        "Value {} outside valid range [{}, {}]",
                        value, rule.min_value, rule.max_value
                    ));
                }
            }
        }

        let processed: Vec<f64> = data
            .iter()
            .map(|&x| {
                let normalized = (x - 50.0) / 50.0;
                normalized.clamp(-1.0, 1.0)
            })
            .collect();

        self.cache.insert(dataset_id.to_string(), processed.clone());
        Ok(processed)
    }

    pub fn get_cached_result(&self, dataset_id: &str) -> Option<&Vec<f64>> {
        self.cache.get(dataset_id)
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    pub fn add_validation_rule(&mut self, rule: ValidationRule) {
        self.validation_rules.push(rule);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_processing() {
        let mut processor = DataProcessor::new();
        let data = vec![25.0, 50.0, 75.0];
        
        let result = processor.process_dataset("test1", &data);
        assert!(result.is_ok());
        
        let processed = result.unwrap();
        assert_eq!(processed.len(), 3);
        assert!(processed[0] >= -1.0 && processed[0] <= 1.0);
    }

    #[test]
    fn test_validation_failure() {
        let mut processor = DataProcessor::new();
        let invalid_data = vec![150.0];
        
        let result = processor.process_dataset("test2", &invalid_data);
        assert!(result.is_err());
    }
}