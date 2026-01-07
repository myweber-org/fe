
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
                let mut valid = true;
                let mut issues = Vec::new();
                
                if rule.required && values.is_empty() {
                    valid = false;
                    issues.push("Required field is empty".to_string());
                }
                
                for (index, &value) in values.iter().enumerate() {
                    if value < rule.min_value || value > rule.max_value {
                        valid = false;
                        issues.push(format!(
                            "Value {} at index {} is outside range [{}, {}]",
                            value, index, rule.min_value, rule.max_value
                        ));
                    }
                }
                
                results.push(ValidationResult {
                    field_name: rule.field_name.clone(),
                    valid,
                    issues,
                });
            }
        }
        
        results
    }

    pub fn normalize_data(&mut self) -> HashMap<String, Vec<f64>> {
        let mut normalized = HashMap::new();
        
        for (name, values) in &self.data {
            if let Some((min, max)) = Self::calculate_min_max(values) {
                let normalized_values: Vec<f64> = values
                    .iter()
                    .map(|&v| (v - min) / (max - min))
                    .collect();
                normalized.insert(name.clone(), normalized_values);
            }
        }
        
        normalized
    }

    fn calculate_min_max(values: &[f64]) -> Option<(f64, f64)> {
        if values.is_empty() {
            return None;
        }
        
        let mut min = values[0];
        let mut max = values[0];
        
        for &value in values.iter().skip(1) {
            if value < min {
                min = value;
            }
            if value > max {
                max = value;
            }
        }
        
        Some((min, max))
    }
}

pub struct ValidationResult {
    field_name: String,
    valid: bool,
    issues: Vec<String>,
}

impl ValidationResult {
    pub fn is_valid(&self) -> bool {
        self.valid
    }
    
    pub fn get_issues(&self) -> &Vec<String> {
        &self.issues
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
    fn test_normalize_data() {
        let mut processor = DataProcessor::new();
        processor.add_dataset("values".to_string(), vec![10.0, 20.0, 30.0]).unwrap();
        
        let normalized = processor.normalize_data();
        let normalized_values = normalized.get("values").unwrap();
        
        assert_eq!(normalized_values[0], 0.0);
        assert_eq!(normalized_values[2], 1.0);
    }
}