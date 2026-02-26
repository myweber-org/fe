
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
            if let Value::Object(obj) = data {
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
            if let Value::Object(obj) = data {
                for (key, value) in obj {
                    if let Some(prop_schema) = properties.get(key) {
                        if let Some(type_str) = prop_schema.get("type").and_then(|v| v.as_str()) {
                            match type_str {
                                "string" => {
                                    if !value.is_string() {
                                        return Ok(false);
                                    }
                                }
                                "number" => {
                                    if !value.is_number() {
                                        return Ok(false);
                                    }
                                }
                                "boolean" => {
                                    if !value.is_boolean() {
                                        return Ok(false);
                                    }
                                }
                                "array" => {
                                    if !value.is_array() {
                                        return Ok(false);
                                    }
                                }
                                "object" => {
                                    if !value.is_object() {
                                        return Ok(false);
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }

        Ok(true)
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
                "name": {"type": "string"},
                "age": {"type": "number"}
            }
        }
        "#;

        let validator = JsonValidator::new(schema).unwrap();
        let json = r#"{"name": "Alice", "age": 30}"#;
        assert!(validator.validate(json).unwrap());
    }

    #[test]
    fn test_missing_required_field() {
        let schema = r#"
        {
            "required": ["name", "age"]
        }
        "#;

        let validator = JsonValidator::new(schema).unwrap();
        let json = r#"{"name": "Bob"}"#;
        assert!(!validator.validate(json).unwrap());
    }

    #[test]
    fn test_type_mismatch() {
        let schema = r#"
        {
            "properties": {
                "age": {"type": "number"}
            }
        }
        "#;

        let validator = JsonValidator::new(schema).unwrap();
        let json = r#"{"age": "thirty"}"#;
        assert!(!validator.validate(json).unwrap());
    }
}