use serde_json::{Value, Map};
use std::collections::HashSet;
use std::error::Error;

pub struct JsonValidator {
    schema: Map<String, Value>,
    required_fields: HashSet<String>,
}

impl JsonValidator {
    pub fn new(schema: Value) -> Result<Self, Box<dyn Error>> {
        let schema_map = schema.as_object().ok_or("Schema must be a JSON object")?.clone();
        let required_fields = Self::extract_required_fields(&schema_map)?;
        Ok(JsonValidator {
            schema: schema_map,
            required_fields,
        })
    }

    fn extract_required_fields(schema: &Map<String, Value>) -> Result<HashSet<String>, Box<dyn Error>> {
        let mut required = HashSet::new();
        if let Some(required_array) = schema.get("required").and_then(|v| v.as_array()) {
            for field in required_array {
                if let Some(field_str) = field.as_str() {
                    required.insert(field_str.to_string());
                }
            }
        }
        Ok(required)
    }

    pub fn validate(&self, data: &Value) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        let data_obj = data.as_object().ok_or_else(|| vec!["Data must be a JSON object".to_string()])?;

        for field in &self.required_fields {
            if !data_obj.contains_key(field) {
                errors.push(format!("Missing required field: {}", field));
            }
        }

        for (key, value) in data_obj {
            if let Some(field_schema) = self.schema.get("properties").and_then(|p| p.get(key)) {
                if let Err(field_errors) = self.validate_field(key, value, field_schema) {
                    errors.extend(field_errors);
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    fn validate_field(&self, field_name: &str, value: &Value, schema: &Value) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        let expected_type = schema.get("type").and_then(|t| t.as_str());

        match expected_type {
            Some("string") => {
                if !value.is_string() {
                    errors.push(format!("Field '{}' must be a string", field_name));
                }
            }
            Some("number") => {
                if !value.is_number() {
                    errors.push(format!("Field '{}' must be a number", field_name));
                }
            }
            Some("boolean") => {
                if !value.is_boolean() {
                    errors.push(format!("Field '{}' must be a boolean", field_name));
                }
            }
            Some("array") => {
                if !value.is_array() {
                    errors.push(format!("Field '{}' must be an array", field_name));
                }
            }
            Some("object") => {
                if !value.is_object() {
                    errors.push(format!("Field '{}' must be an object", field_name));
                }
            }
            _ => {}
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_valid_json() {
        let schema = json!({
            "type": "object",
            "required": ["name", "age"],
            "properties": {
                "name": {"type": "string"},
                "age": {"type": "number"},
                "active": {"type": "boolean"}
            }
        });

        let validator = JsonValidator::new(schema).unwrap();
        let data = json!({
            "name": "John",
            "age": 30,
            "active": true
        });

        assert!(validator.validate(&data).is_ok());
    }

    #[test]
    fn test_missing_required_field() {
        let schema = json!({
            "type": "object",
            "required": ["name", "age"],
            "properties": {
                "name": {"type": "string"},
                "age": {"type": "number"}
            }
        });

        let validator = JsonValidator::new(schema).unwrap();
        let data = json!({
            "name": "John"
        });

        let result = validator.validate(&data);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.contains("Missing required field: age")));
    }

    #[test]
    fn test_type_mismatch() {
        let schema = json!({
            "type": "object",
            "properties": {
                "age": {"type": "number"}
            }
        });

        let validator = JsonValidator::new(schema).unwrap();
        let data = json!({
            "age": "thirty"
        });

        let result = validator.validate(&data);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.contains("must be a number")));
    }
}