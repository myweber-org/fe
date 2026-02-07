
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

    pub fn add_dataset(&mut self, name: String, values: Vec<f64>) {
        self.data.insert(name, values);
    }

    pub fn add_validation_rule(&mut self, rule: ValidationRule) {
        self.validation_rules.push(rule);
    }

    pub fn validate_all(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        for rule in &self.validation_rules {
            match self.data.get(&rule.field_name) {
                Some(values) => {
                    if rule.required && values.is_empty() {
                        errors.push(format!("Field '{}' is required but empty", rule.field_name));
                    }

                    for (index, &value) in values.iter().enumerate() {
                        if value < rule.min_value || value > rule.max_value {
                            errors.push(format!(
                                "Value {} at index {} in field '{}' is out of range [{}, {}]",
                                value, index, rule.field_name, rule.min_value, rule.max_value
                            ));
                        }
                    }
                }
                None if rule.required => {
                    errors.push(format!("Required field '{}' not found", rule.field_name));
                }
                None => {}
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    pub fn normalize_data(&mut self) {
        for values in self.data.values_mut() {
            if let Some(&max) = values.iter().max_by(|a, b| a.partial_cmp(b).unwrap()) {
                if max != 0.0 {
                    for value in values.iter_mut() {
                        *value /= max;
                    }
                }
            }
        }
    }

    pub fn calculate_statistics(&self) -> HashMap<String, Statistics> {
        let mut stats = HashMap::new();

        for (name, values) in &self.data {
            if values.is_empty() {
                continue;
            }

            let sum: f64 = values.iter().sum();
            let count = values.len() as f64;
            let mean = sum / count;

            let variance: f64 = values.iter()
                .map(|&x| (x - mean).powi(2))
                .sum::<f64>() / count;

            stats.insert(name.clone(), Statistics {
                mean,
                variance,
                min: *values.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap(),
                max: *values.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap(),
                count: values.len(),
            });
        }

        stats
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
    pub fn new(field_name: String, min_value: f64, max_value: f64, required: bool) -> Self {
        ValidationRule {
            field_name,
            min_value,
            max_value,
            required,
        }
    }
}