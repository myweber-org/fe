use serde_json::{Value, from_str};
use std::collections::HashSet;
use std::error::Error;

pub struct JsonValidator {
    required_fields: HashSet<String>,
    allowed_types: HashSet<&'static str>,
}

impl JsonValidator {
    pub fn new() -> Self {
        JsonValidator {
            required_fields: HashSet::new(),
            allowed_types: HashSet::from(["object", "array", "string", "number", "boolean", "null"]),
        }
    }

    pub fn add_required_field(&mut self, field: &str) {
        self.required_fields.insert(field.to_string());
    }

    pub fn validate(&self, json_str: &str) -> Result<Value, Box<dyn Error>> {
        let parsed: Value = from_str(json_str)?;
        
        if let Value::Object(ref obj) = parsed {
            for field in &self.required_fields {
                if !obj.contains_key(field) {
                    return Err(format!("Missing required field: {}", field).into());
                }
            }
        }

        self.validate_value_type(&parsed)?;
        Ok(parsed)
    }

    fn validate_value_type(&self, value: &Value) -> Result<(), Box<dyn Error>> {
        let type_name = match value {
            Value::Object(_) => "object",
            Value::Array(_) => "array",
            Value::String(_) => "string",
            Value::Number(_) => "number",
            Value::Bool(_) => "boolean",
            Value::Null => "null",
        };

        if !self.allowed_types.contains(type_name) {
            return Err(format!("Disallowed JSON type: {}", type_name).into());
        }

        Ok(())
    }

    pub fn restrict_types(&mut self, types: &[&'static str]) {
        self.allowed_types.clear();
        for &t in types {
            self.allowed_types.insert(t);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_validation() {
        let mut validator = JsonValidator::new();
        validator.add_required_field("id");
        validator.add_required_field("name");
        
        let valid_json = r#"{"id": 1, "name": "test"}"#;
        assert!(validator.validate(valid_json).is_ok());
        
        let invalid_json = r#"{"id": 1}"#;
        assert!(validator.validate(invalid_json).is_err());
    }

    #[test]
    fn test_type_restriction() {
        let mut validator = JsonValidator::new();
        validator.restrict_types(&["object", "string"]);
        
        assert!(validator.validate(r#"{"key": "value"}"#).is_ok());
        assert!(validator.validate(r#""simple string""#).is_ok());
        assert!(validator.validate(r#"[1, 2, 3]"#).is_err());
    }
}