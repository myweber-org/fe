
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

    pub fn add_dataset(&mut self, name: String, values: Vec<f64>) -> Result<(), String> {
        if name.is_empty() {
            return Err("Dataset name cannot be empty".to_string());
        }
        
        if values.is_empty() {
            return Err("Dataset values cannot be empty".to_string());
        }

        self.data.insert(name, values);
        Ok(())
    }

    pub fn add_validation_rule(&mut self, rule: ValidationRule) {
        self.validation_rules.push(rule);
    }

    pub fn validate_data(&self) -> Vec<ValidationResult> {
        let mut results = Vec::new();

        for rule in &self.validation_rules {
            if let Some(data_values) = self.data.get(&rule.field_name) {
                if rule.required && data_values.is_empty() {
                    results.push(ValidationResult::new(
                        &rule.field_name,
                        false,
                        "Required field is empty"
                    ));
                    continue;
                }

                let mut valid = true;
                let mut message = String::new();

                for &value in data_values {
                    if value < rule.min_value || value > rule.max_value {
                        valid = false;
                        message = format!("Value {} out of range [{}, {}]", 
                                         value, rule.min_value, rule.max_value);
                        break;
                    }
                }

                results.push(ValidationResult::new(
                    &rule.field_name,
                    valid,
                    if valid { "Validation passed" } else { &message }
                ));
            } else if rule.required {
                results.push(ValidationResult::new(
                    &rule.field_name,
                    false,
                    "Required field not found"
                ));
            }
        }

        results
    }

    pub fn calculate_statistics(&self, dataset_name: &str) -> Option<DatasetStatistics> {
        self.data.get(dataset_name).map(|values| {
            let count = values.len();
            let sum: f64 = values.iter().sum();
            let mean = if count > 0 { sum / count as f64 } else { 0.0 };
            
            let variance: f64 = if count > 1 {
                values.iter()
                    .map(|&x| (x - mean).powi(2))
                    .sum::<f64>() / (count - 1) as f64
            } else {
                0.0
            };

            let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
            let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

            DatasetStatistics {
                count,
                mean,
                variance,
                standard_deviation: variance.sqrt(),
                min,
                max,
            }
        })
    }

    pub fn normalize_data(&mut self, dataset_name: &str) -> Result<(), String> {
        if let Some(values) = self.data.get_mut(dataset_name) {
            if values.is_empty() {
                return Err("Cannot normalize empty dataset".to_string());
            }

            let stats = self.calculate_statistics(dataset_name).unwrap();
            
            if stats.standard_deviation == 0.0 {
                return Err("Cannot normalize dataset with zero standard deviation".to_string());
            }

            for value in values.iter_mut() {
                *value = (*value - stats.mean) / stats.standard_deviation;
            }

            Ok(())
        } else {
            Err(format!("Dataset '{}' not found", dataset_name))
        }
    }
}

pub struct ValidationResult {
    field_name: String,
    is_valid: bool,
    message: String,
}

impl ValidationResult {
    pub fn new(field_name: &str, is_valid: bool, message: &str) -> Self {
        ValidationResult {
            field_name: field_name.to_string(),
            is_valid,
            message: message.to_string(),
        }
    }

    pub fn is_valid(&self) -> bool {
        self.is_valid
    }

    pub fn get_message(&self) -> &str {
        &self.message
    }
}

pub struct DatasetStatistics {
    pub count: usize,
    pub mean: f64,
    pub variance: f64,
    pub standard_deviation: f64,
    pub min: f64,
    pub max: f64,
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
    fn test_add_dataset() {
        let mut processor = DataProcessor::new();
        let result = processor.add_dataset("test_data".to_string(), vec![1.0, 2.0, 3.0]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_empty_dataset_name() {
        let mut processor = DataProcessor::new();
        let result = processor.add_dataset("".to_string(), vec![1.0, 2.0]);
        assert!(result.is_err());
    }

    #[test]
    fn test_calculate_statistics() {
        let mut processor = DataProcessor::new();
        processor.add_dataset("numbers".to_string(), vec![1.0, 2.0, 3.0, 4.0, 5.0]).unwrap();
        
        let stats = processor.calculate_statistics("numbers").unwrap();
        assert_eq!(stats.count, 5);
        assert_eq!(stats.mean, 3.0);
        assert_eq!(stats.min, 1.0);
        assert_eq!(stats.max, 5.0);
    }

    #[test]
    fn test_validation() {
        let mut processor = DataProcessor::new();
        processor.add_dataset("temperature".to_string(), vec![20.5, 22.1, 19.8]).unwrap();
        
        let rule = ValidationRule::new("temperature".to_string(), 15.0, 30.0, true);
        processor.add_validation_rule(rule);
        
        let results = processor.validate_data();
        assert_eq!(results.len(), 1);
        assert!(results[0].is_valid());
    }
}