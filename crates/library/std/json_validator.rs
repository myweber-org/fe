use serde_json::{Value, Map};
use std::collections::HashSet;
use std::error::Error;

pub struct JsonValidator {
    required_fields: HashSet<String>,
    allowed_types: Map<String, String>,
}

impl JsonValidator {
    pub fn new() -> Self {
        JsonValidator {
            required_fields: HashSet::new(),
            allowed_types: Map::new(),
        }
    }

    pub fn add_required_field(&mut self, field: &str) {
        self.required_fields.insert(field.to_string());
    }

    pub fn add_type_requirement(&mut self, field: &str, expected_type: &str) {
        self.allowed_types.insert(field.to_string(), expected_type.to_string());
    }

    pub fn validate(&self, json_str: &str) -> Result<(), Box<dyn Error>> {
        let parsed: Value = serde_json::from_str(json_str)?;
        
        if let Value::Object(obj) = &parsed {
            for field in &self.required_fields {
                if !obj.contains_key(field) {
                    return Err(format!("Missing required field: {}", field).into());
                }
            }

            for (field, expected_type) in &self.allowed_types {
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
                        return Err(format!("Field '{}' should be {}, got {}", field, expected_type, actual_type).into());
                    }
                }
            }
        } else {
            return Err("Expected JSON object".into());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_json() {
        let mut validator = JsonValidator::new();
        validator.add_required_field("name");
        validator.add_type_requirement("age", "number");

        let json = r#"{"name": "John", "age": 30}"#;
        assert!(validator.validate(json).is_ok());
    }

    #[test]
    fn test_missing_field() {
        let mut validator = JsonValidator::new();
        validator.add_required_field("email");

        let json = r#"{"name": "John"}"#;
        assert!(validator.validate(json).is_err());
    }

    #[test]
    fn test_type_mismatch() {
        let mut validator = JsonValidator::new();
        validator.add_type_requirement("count", "number");

        let json = r#"{"count": "five"}"#;
        assert!(validator.validate(json).is_err());
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
    
    match compiled_schema.validate(&data_value) {
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