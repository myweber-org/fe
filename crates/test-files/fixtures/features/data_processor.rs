
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

    pub fn validate_all(&self) -> Vec<ValidationResult> {
        let mut results = Vec::new();
        
        for rule in &self.validation_rules {
            if let Some(values) = self.data.get(&rule.field_name) {
                let result = self.validate_dataset(values, rule);
                results.push(result);
            } else if rule.required {
                results.push(ValidationResult {
                    field_name: rule.field_name.clone(),
                    valid: false,
                    message: "Required field not found".to_string(),
                });
            }
        }
        
        results
    }

    fn validate_dataset(&self, values: &[f64], rule: &ValidationRule) -> ValidationResult {
        let mut invalid_count = 0;
        
        for &value in values {
            if value < rule.min_value || value > rule.max_value {
                invalid_count += 1;
            }
        }

        let valid = invalid_count == 0;
        let message = if valid {
            "All values within valid range".to_string()
        } else {
            format!("{} values out of range", invalid_count)
        };

        ValidationResult {
            field_name: rule.field_name.clone(),
            valid,
            message,
        }
    }

    pub fn calculate_statistics(&self, dataset_name: &str) -> Option<Statistics> {
        self.data.get(dataset_name).map(|values| {
            let sum: f64 = values.iter().sum();
            let count = values.len() as f64;
            let mean = sum / count;
            
            let variance: f64 = values.iter()
                .map(|&x| (x - mean).powi(2))
                .sum::<f64>() / count;
            
            let std_dev = variance.sqrt();
            
            let mut sorted_values = values.clone();
            sorted_values.sort_by(|a, b| a.partial_cmp(b).unwrap());
            
            let median = if count as usize % 2 == 0 {
                let mid = count as usize / 2;
                (sorted_values[mid - 1] + sorted_values[mid]) / 2.0
            } else {
                sorted_values[count as usize / 2]
            };

            Statistics {
                mean,
                median,
                std_dev,
                min: *sorted_values.first().unwrap_or(&0.0),
                max: *sorted_values.last().unwrap_or(&0.0),
                count: values.len(),
            }
        })
    }

    pub fn normalize_data(&mut self, dataset_name: &str) -> Result<Vec<f64>, String> {
        if let Some(values) = self.data.get_mut(dataset_name) {
            let stats = self.calculate_statistics(dataset_name)
                .ok_or_else(|| "Failed to calculate statistics".to_string())?;
            
            if stats.std_dev == 0.0 {
                return Err("Cannot normalize data with zero standard deviation".to_string());
            }

            let normalized: Vec<f64> = values.iter()
                .map(|&x| (x - stats.mean) / stats.std_dev)
                .collect();
            
            *values = normalized.clone();
            Ok(normalized)
        } else {
            Err(format!("Dataset '{}' not found", dataset_name))
        }
    }
}

pub struct ValidationResult {
    field_name: String,
    valid: bool,
    message: String,
}

pub struct Statistics {
    mean: f64,
    median: f64,
    std_dev: f64,
    min: f64,
    max: f64,
    count: usize,
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
        processor.add_dataset("test".to_string(), vec![1.0, 2.0, 3.0, 4.0, 5.0]).unwrap();
        
        let stats = processor.calculate_statistics("test").unwrap();
        assert_eq!(stats.mean, 3.0);
        assert_eq!(stats.median, 3.0);
        assert_eq!(stats.count, 5);
    }

    #[test]
    fn test_validation() {
        let mut processor = DataProcessor::new();
        processor.add_dataset("temperature".to_string(), vec![20.0, 25.0, 30.0]).unwrap();
        
        let rule = ValidationRule::new("temperature".to_string(), 15.0, 35.0, true);
        processor.add_validation_rule(rule);
        
        let results = processor.validate_all();
        assert_eq!(results.len(), 1);
        assert!(results[0].valid);
    }
}