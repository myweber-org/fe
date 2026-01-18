use serde_json::{Value, Map};
use std::collections::HashSet;
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct ValidationError {
    field: String,
    message: String,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Field '{}': {}", self.field, self.message)
    }
}

impl Error for ValidationError {}

pub struct JsonValidator {
    required_fields: HashSet<String>,
    field_types: Map<String, String>,
    custom_rules: Map<String, Box<dyn Fn(&Value) -> Result<(), ValidationError>>>,
}

impl JsonValidator {
    pub fn new() -> Self {
        JsonValidator {
            required_fields: HashSet::new(),
            field_types: Map::new(),
            custom_rules: Map::new(),
        }
    }

    pub fn require_field(mut self, field: &str) -> Self {
        self.required_fields.insert(field.to_string());
        self
    }

    pub fn expect_type(mut self, field: &str, type_name: &str) -> Self {
        self.field_types.insert(field.to_string(), type_name.to_string());
        self
    }

    pub fn add_rule<F>(mut self, field: &str, rule: F) -> Self
    where
        F: Fn(&Value) -> Result<(), ValidationError> + 'static,
    {
        self.custom_rules.insert(field.to_string(), Box::new(rule));
        self
    }

    pub fn validate(&self, data: &Value) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();

        if let Value::Object(obj) = data {
            for field in &self.required_fields {
                if !obj.contains_key(field) {
                    errors.push(ValidationError {
                        field: field.clone(),
                        message: "This field is required".to_string(),
                    });
                }
            }

            for (field, expected_type) in &self.field_types {
                if let Some(value) = obj.get(field) {
                    let actual_type = match value {
                        Value::Null => "null",
                        Value::Bool(_) => "boolean",
                        Value::Number(_) => "number",
                        Value::String(_) => "string",
                        Value::Array(_) => "array",
                        Value::Object(_) => "object",
                    };

                    if actual_type != expected_type {
                        errors.push(ValidationError {
                            field: field.clone(),
                            message: format!("Expected type '{}', got '{}'", expected_type, actual_type),
                        });
                    }
                }
            }

            for (field, rule) in &self.custom_rules {
                if let Some(value) = obj.get(field) {
                    if let Err(err) = rule(value) {
                        errors.push(err);
                    }
                }
            }
        } else {
            errors.push(ValidationError {
                field: "root".to_string(),
                message: "Expected JSON object".to_string(),
            });
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

pub fn validate_email(value: &Value) -> Result<(), ValidationError> {
    if let Value::String(email) = value {
        if email.contains('@') && email.contains('.') {
            Ok(())
        } else {
            Err(ValidationError {
                field: "email".to_string(),
                message: "Invalid email format".to_string(),
            })
        }
    } else {
        Err(ValidationError {
            field: "email".to_string(),
            message: "Expected string value".to_string(),
        })
    }
}

pub fn validate_age(value: &Value) -> Result<(), ValidationError> {
    if let Value::Number(num) = value {
        if let Some(age) = num.as_u64() {
            if age >= 18 && age <= 120 {
                Ok(())
            } else {
                Err(ValidationError {
                    field: "age".to_string(),
                    message: "Age must be between 18 and 120".to_string(),
                })
            }
        } else {
            Err(ValidationError {
                field: "age".to_string(),
                message: "Age must be a positive integer".to_string(),
            })
        }
    } else {
        Err(ValidationError {
            field: "age".to_string(),
            message: "Expected number value".to_string(),
        })
    }
}