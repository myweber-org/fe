use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct UserData {
    pub username: Option<String>,
    pub email: Option<String>,
    pub age: Option<u32>,
}

pub struct Validator {
    rules: HashMap<String, ValidationRule>,
}

pub struct ValidationRule {
    required: bool,
    min_length: Option<usize>,
    max_length: Option<usize>,
    validator_fn: Option<Box<dyn Fn(&str) -> bool>>,
}

impl Validator {
    pub fn new() -> Self {
        Validator {
            rules: HashMap::new(),
        }
    }

    pub fn add_rule(&mut self, field: &str, rule: ValidationRule) {
        self.rules.insert(field.to_string(), rule);
    }

    pub fn validate(&self, data: &UserData) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        for (field, rule) in &self.rules {
            if rule.required {
                match field.as_str() {
                    "username" => {
                        if data.username.is_none() || data.username.as_ref().unwrap().is_empty() {
                            errors.push(format!("{} is required", field));
                        }
                    }
                    "email" => {
                        if data.email.is_none() || data.email.as_ref().unwrap().is_empty() {
                            errors.push(format!("{} is required", field));
                        }
                    }
                    "age" => {
                        if data.age.is_none() {
                            errors.push(format!("{} is required", field));
                        }
                    }
                    _ => {}
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl ValidationRule {
    pub fn required() -> Self {
        ValidationRule {
            required: true,
            min_length: None,
            max_length: None,
            validator_fn: None,
        }
    }

    pub fn with_min_length(mut self, length: usize) -> Self {
        self.min_length = Some(length);
        self
    }

    pub fn with_max_length(mut self, length: usize) -> Self {
        self.max_length = Some(length);
        self
    }

    pub fn with_validator<F>(mut self, validator: F) -> Self
    where
        F: Fn(&str) -> bool + 'static,
    {
        self.validator_fn = Some(Box::new(validator));
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_required_field_validation() {
        let mut validator = Validator::new();
        validator.add_rule("username", ValidationRule::required());
        validator.add_rule("email", ValidationRule::required());

        let valid_data = UserData {
            username: Some("john_doe".to_string()),
            email: Some("john@example.com".to_string()),
            age: Some(25),
        };

        let invalid_data = UserData {
            username: None,
            email: Some("".to_string()),
            age: Some(30),
        };

        assert!(validator.validate(&valid_data).is_ok());
        assert!(validator.validate(&invalid_data).is_err());
    }
}