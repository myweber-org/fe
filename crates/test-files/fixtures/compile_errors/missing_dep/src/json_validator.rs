use serde_json::{Value, from_str};
use std::fs;

pub struct JsonValidator {
    schema: Value,
}

impl JsonValidator {
    pub fn new(schema_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let schema_content = fs::read_to_string(schema_path)?;
        let schema: Value = from_str(&schema_content)?;
        Ok(JsonValidator { schema })
    }

    pub fn validate(&self, json_str: &str) -> Result<bool, Box<dyn std::error::Error>> {
        let data: Value = from_str(json_str)?;
        self.validate_value(&data)
    }

    fn validate_value(&self, data: &Value) -> Result<bool, Box<dyn std::error::Error>> {
        if self.schema["type"] == "object" {
            self.validate_object(data)
        } else if self.schema["type"] == "array" {
            self.validate_array(data)
        } else {
            self.validate_primitive(data)
        }
    }

    fn validate_object(&self, data: &Value) -> Result<bool, Box<dyn std::error::Error>> {
        if !data.is_object() {
            return Ok(false);
        }

        let obj = data.as_object().unwrap();
        let required_fields = self.schema["required"].as_array();

        if let Some(required) = required_fields {
            for field in required {
                let field_name = field.as_str().unwrap();
                if !obj.contains_key(field_name) {
                    return Ok(false);
                }
            }
        }

        Ok(true)
    }

    fn validate_array(&self, data: &Value) -> Result<bool, Box<dyn std::error::Error>> {
        if !data.is_array() {
            return Ok(false);
        }

        let arr = data.as_array().unwrap();
        if let Some(min_items) = self.schema["minItems"].as_u64() {
            if arr.len() < min_items as usize {
                return Ok(false);
            }
        }

        if let Some(max_items) = self.schema["maxItems"].as_u64() {
            if arr.len() > max_items as usize {
                return Ok(false);
            }
        }

        Ok(true)
    }

    fn validate_primitive(&self, data: &Value) -> Result<bool, Box<dyn std::error::Error>> {
        let schema_type = self.schema["type"].as_str().unwrap_or("string");
        
        match schema_type {
            "string" => Ok(data.is_string()),
            "number" => Ok(data.is_number()),
            "integer" => Ok(data.is_i64() || data.is_u64()),
            "boolean" => Ok(data.is_boolean()),
            "null" => Ok(data.is_null()),
            _ => Ok(false),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_object_validation() {
        let schema = r#"{
            "type": "object",
            "required": ["name", "age"]
        }"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", schema).unwrap();
        
        let validator = JsonValidator::new(temp_file.path().to_str().unwrap()).unwrap();
        
        let valid_json = r#"{"name": "John", "age": 30}"#;
        assert!(validator.validate(valid_json).unwrap());
        
        let invalid_json = r#"{"name": "John"}"#;
        assert!(!validator.validate(invalid_json).unwrap());
    }

    #[test]
    fn test_array_validation() {
        let schema = r#"{
            "type": "array",
            "minItems": 2,
            "maxItems": 4
        }"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", schema).unwrap();
        
        let validator = JsonValidator::new(temp_file.path().to_str().unwrap()).unwrap();
        
        let valid_json = r#"[1, 2, 3]"#;
        assert!(validator.validate(valid_json).unwrap());
        
        let invalid_json = r#"[1]"#;
        assert!(!validator.validate(invalid_json).unwrap());
    }
}