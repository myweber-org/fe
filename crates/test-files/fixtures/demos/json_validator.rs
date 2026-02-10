use serde_json::{Value, Map};
use std::collections::HashSet;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct ValidationError {
    pub path: String,
    pub message: String,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.path, self.message)
    }
}

impl Error for ValidationError {}

pub struct JsonValidator {
    required_fields: HashSet<String>,
    field_types: Map<String, String>,
    custom_validators: Map<String, Box<dyn Fn(&Value) -> Result<(), ValidationError>>>,
}

impl JsonValidator {
    pub fn new() -> Self {
        JsonValidator {
            required_fields: HashSet::new(),
            field_types: Map::new(),
            custom_validators: Map::new(),
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

    pub fn add_validator<F>(mut self, field: &str, validator: F) -> Self
    where
        F: Fn(&Value) -> Result<(), ValidationError> + 'static,
    {
        self.custom_validators.insert(field.to_string(), Box::new(validator));
        self
    }

    pub fn validate(&self, data: &Value) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();

        if let Value::Object(obj) = data {
            self.validate_required_fields(obj, &mut errors);
            self.validate_field_types(obj, &mut errors);
            self.run_custom_validators(obj, &mut errors);
        } else {
            errors.push(ValidationError {
                path: "".to_string(),
                message: "Expected JSON object".to_string(),
            });
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    fn validate_required_fields(&self, obj: &Map<String, Value>, errors: &mut Vec<ValidationError>) {
        for field in &self.required_fields {
            if !obj.contains_key(field) {
                errors.push(ValidationError {
                    path: field.clone(),
                    message: "Required field is missing".to_string(),
                });
            }
        }
    }

    fn validate_field_types(&self, obj: &Map<String, Value>, errors: &mut Vec<ValidationError>) {
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
                        path: field.clone(),
                        message: format!("Expected type '{}', got '{}'", expected_type, actual_type),
                    });
                }
            }
        }
    }

    fn run_custom_validators(&self, obj: &Map<String, Value>, errors: &mut Vec<ValidationError>) {
        for (field, validator) in &self.custom_validators {
            if let Some(value) = obj.get(field) {
                if let Err(err) = validator(value) {
                    errors.push(err);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_basic_validation() {
        let validator = JsonValidator::new()
            .require_field("name")
            .expect_type("age", "number")
            .add_validator("age", |v| {
                if let Some(age) = v.as_i64() {
                    if age < 0 || age > 150 {
                        Err(ValidationError {
                            path: "age".to_string(),
                            message: "Age must be between 0 and 150".to_string(),
                        })
                    } else {
                        Ok(())
                    }
                } else {
                    Ok(())
                }
            });

        let valid_data = json!({
            "name": "John",
            "age": 30
        });

        let invalid_data = json!({
            "name": "John",
            "age": -5
        });

        assert!(validator.validate(&valid_data).is_ok());
        assert!(validator.validate(&invalid_data).is_err());
    }
}