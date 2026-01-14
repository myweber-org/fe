
use serde_json::{Value, from_str};
use std::error::Error;

pub struct JsonValidator {
    schema: Value,
}

impl JsonValidator {
    pub fn new(schema_str: &str) -> Result<Self, Box<dyn Error>> {
        let schema = from_str(schema_str)?;
        Ok(JsonValidator { schema })
    }

    pub fn validate(&self, json_str: &str) -> Result<bool, Box<dyn Error>> {
        let data: Value = from_str(json_str)?;
        self.validate_value(&data)
    }

    fn validate_value(&self, data: &Value) -> Result<bool, Box<dyn Error>> {
        if let Some(required_fields) = self.schema.get("required").and_then(|v| v.as_array()) {
            if let Some(obj) = data.as_object() {
                for field in required_fields {
                    if let Some(field_name) = field.as_str() {
                        if !obj.contains_key(field_name) {
                            return Ok(false);
                        }
                    }
                }
            }
        }

        if let Some(properties) = self.schema.get("properties").and_then(|v| v.as_object()) {
            if let Some(obj) = data.as_object() {
                for (key, value) in obj {
                    if let Some(prop_schema) = properties.get(key) {
                        if !self.validate_type(value, prop_schema)? {
                            return Ok(false);
                        }
                    }
                }
            }
        }

        Ok(true)
    }

    fn validate_type(&self, value: &Value, schema: &Value) -> Result<bool, Box<dyn Error>> {
        if let Some(type_str) = schema.get("type").and_then(|v| v.as_str()) {
            match type_str {
                "string" => Ok(value.is_string()),
                "number" => Ok(value.is_number()),
                "integer" => Ok(value.is_i64() || value.is_u64()),
                "boolean" => Ok(value.is_boolean()),
                "array" => Ok(value.is_array()),
                "object" => Ok(value.is_object()),
                _ => Ok(false),
            }
        } else {
            Ok(true)
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
            "required": ["name", "age"],
            "properties": {
                "name": { "type": "string" },
                "age": { "type": "integer" }
            }
        }
        "#;

        let validator = JsonValidator::new(schema).unwrap();
        let json = r#"{"name": "Alice", "age": 30}"#;
        assert!(validator.validate(json).unwrap());
    }

    #[test]
    fn test_invalid_type() {
        let schema = r#"
        {
            "type": "object",
            "properties": {
                "count": { "type": "integer" }
            }
        }
        "#;

        let validator = JsonValidator::new(schema).unwrap();
        let json = r#"{"count": "not_a_number"}"#;
        assert!(!validator.validate(json).unwrap());
    }
}