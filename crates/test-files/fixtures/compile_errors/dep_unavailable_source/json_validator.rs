use serde_json::{Value, json};
use std::collections::HashSet;

pub struct JsonValidator {
    required_fields: HashSet<String>,
    allowed_types: HashSet<&'static str>,
}

impl JsonValidator {
    pub fn new() -> Self {
        JsonValidator {
            required_fields: HashSet::new(),
            allowed_types: HashSet::from(["string", "number", "boolean", "object", "array", "null"]),
        }
    }

    pub fn add_required_field(&mut self, field: &str) {
        self.required_fields.insert(field.to_string());
    }

    pub fn validate(&self, json_str: &str) -> Result<Value, String> {
        let parsed: Value = serde_json::from_str(json_str)
            .map_err(|e| format!("Invalid JSON: {}", e))?;

        self.validate_structure(&parsed)?;
        Ok(parsed)
    }

    fn validate_structure(&self, value: &Value) -> Result<(), String> {
        match value {
            Value::Object(map) => {
                for field in &self.required_fields {
                    if !map.contains_key(field) {
                        return Err(format!("Missing required field: {}", field));
                    }
                }

                for (key, val) in map {
                    if !self.is_valid_type(val) {
                        return Err(format!("Field '{}' has invalid type", key));
                    }
                    self.validate_structure(val)?;
                }
            }
            Value::Array(arr) => {
                for item in arr {
                    self.validate_structure(item)?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn is_valid_type(&self, value: &Value) -> bool {
        let type_str = match value {
            Value::String(_) => "string",
            Value::Number(_) => "number",
            Value::Bool(_) => "boolean",
            Value::Object(_) => "object",
            Value::Array(_) => "array",
            Value::Null => "null",
        };
        self.allowed_types.contains(type_str)
    }
}

pub fn create_sample_schema() -> Value {
    json!({
        "name": "Test User",
        "age": 30,
        "active": true,
        "metadata": {
            "role": "admin",
            "permissions": ["read", "write"]
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_json() {
        let mut validator = JsonValidator::new();
        validator.add_required_field("name");
        
        let json_data = r#"{"name": "John", "age": 25}"#;
        assert!(validator.validate(json_data).is_ok());
    }

    #[test]
    fn test_missing_required_field() {
        let mut validator = JsonValidator::new();
        validator.add_required_field("email");
        
        let json_data = r#"{"name": "John"}"#;
        assert!(validator.validate(json_data).is_err());
    }
}