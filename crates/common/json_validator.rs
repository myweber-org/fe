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
}use serde_json::Value;
use jsonschema::JSONSchema;

pub fn validate_json(schema: &str, data: &str) -> Result<(), Vec<String>> {
    let schema_value: Value = serde_json::from_str(schema)
        .map_err(|e| vec![format!("Invalid schema: {}", e)])?;
    
    let data_value: Value = serde_json::from_str(data)
        .map_err(|e| vec![format!("Invalid JSON data: {}", e)])?;
    
    let compiled_schema = JSONSchema::compile(&schema_value)
        .map_err(|e| vec![format!("Schema compilation failed: {}", e)])?;
    
    let validation_result = compiled_schema.validate(&data_value);
    
    match validation_result {
        Ok(_) => Ok(()),
        Err(errors) => {
            let error_messages: Vec<String> = errors
                .map(|e| format!("Validation error: {}", e))
                .collect();
            Err(error_messages)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_json() {
        let schema = r#"
        {
            "type": "object",
            "properties": {
                "name": {"type": "string"},
                "age": {"type": "number"}
            },
            "required": ["name"]
        }
        "#;

        let valid_data = r#"{"name": "Alice", "age": 30}"#;
        assert!(validate_json(schema, valid_data).is_ok());
    }

    #[test]
    fn test_invalid_json() {
        let schema = r#"
        {
            "type": "object",
            "properties": {
                "name": {"type": "string"}
            },
            "required": ["name"]
        }
        "#;

        let invalid_data = r#"{"age": 30}"#;
        assert!(validate_json(schema, invalid_data).is_err());
    }
}