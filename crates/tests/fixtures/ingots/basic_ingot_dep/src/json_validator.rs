use serde_json::{Value, json};
use std::collections::HashSet;

pub struct JsonValidator {
    required_fields: HashSet<String>,
    allowed_types: HashSet<String>,
}

impl JsonValidator {
    pub fn new() -> Self {
        JsonValidator {
            required_fields: HashSet::new(),
            allowed_types: HashSet::new(),
        }
    }

    pub fn add_required_field(&mut self, field: &str) {
        self.required_fields.insert(field.to_string());
    }

    pub fn add_allowed_type(&mut self, type_name: &str) {
        self.allowed_types.insert(type_name.to_string());
    }

    pub fn validate(&self, json_str: &str) -> Result<Value, String> {
        let parsed: Value = serde_json::from_str(json_str)
            .map_err(|e| format!("Invalid JSON: {}", e))?;

        self.validate_structure(&parsed)?;
        self.validate_types(&parsed)?;

        Ok(parsed)
    }

    fn validate_structure(&self, value: &Value) -> Result<(), String> {
        if let Value::Object(map) = value {
            for field in &self.required_fields {
                if !map.contains_key(field) {
                    return Err(format!("Missing required field: {}", field));
                }
            }
            Ok(())
        } else {
            Err("Expected JSON object".to_string())
        }
    }

    fn validate_types(&self, value: &Value) -> Result<(), String> {
        if self.allowed_types.is_empty() {
            return Ok(());
        }

        if let Value::Object(map) = value {
            for (key, val) in map {
                let type_name = match val {
                    Value::Null => "null",
                    Value::Bool(_) => "boolean",
                    Value::Number(_) => "number",
                    Value::String(_) => "string",
                    Value::Array(_) => "array",
                    Value::Object(_) => "object",
                };

                if !self.allowed_types.contains(type_name) {
                    return Err(format!("Field '{}' has disallowed type: {}", key, type_name));
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_validation() {
        let mut validator = JsonValidator::new();
        validator.add_required_field("name");
        validator.add_allowed_type("string");
        validator.add_allowed_type("number");

        let valid_json = r#"{"name": "test", "age": 25}"#;
        let result = validator.validate(valid_json);
        assert!(result.is_ok());

        let invalid_json = r#"{"age": 25}"#;
        let result = validator.validate(invalid_json);
        assert!(result.is_err());
    }
}