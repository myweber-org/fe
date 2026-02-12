use serde_json::{Value, Map};
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct ValidationError {
    message: String,
    path: String,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Validation failed at {}: {}", self.path, self.message)
    }
}

impl Error for ValidationError {}

pub struct JsonValidator {
    schema: Map<String, Value>,
    strict_mode: bool,
}

impl JsonValidator {
    pub fn new(schema: Value) -> Result<Self, Box<dyn Error>> {
        if !schema.is_object() {
            return Err("Schema must be a JSON object".into());
        }
        
        Ok(JsonValidator {
            schema: schema.as_object().unwrap().clone(),
            strict_mode: false,
        })
    }
    
    pub fn with_strict_mode(mut self, strict: bool) -> Self {
        self.strict_mode = strict;
        self
    }
    
    pub fn validate(&self, data: &Value) -> Result<(), ValidationError> {
        self.validate_object(&self.schema, data, "")
    }
    
    fn validate_object(
        &self,
        schema: &Map<String, Value>,
        data: &Value,
        path: &str,
    ) -> Result<(), ValidationError> {
        if !data.is_object() {
            return Err(ValidationError {
                message: "Expected object".to_string(),
                path: path.to_string(),
            });
        }
        
        let data_obj = data.as_object().unwrap();
        
        for (key, prop_schema) in schema {
            let new_path = if path.is_empty() {
                key.clone()
            } else {
                format!("{}.{}", path, key)
            };
            
            match data_obj.get(key) {
                Some(value) => {
                    self.validate_value(prop_schema, value, &new_path)?;
                }
                None => {
                    if self.strict_mode || prop_schema.get("required").and_then(|v| v.as_bool()).unwrap_or(false) {
                        return Err(ValidationError {
                            message: format!("Missing required field: {}", key),
                            path: new_path,
                        });
                    }
                }
            }
        }
        
        Ok(())
    }
    
    fn validate_value(
        &self,
        schema: &Value,
        data: &Value,
        path: &str,
    ) -> Result<(), ValidationError> {
        match schema.get("type") {
            Some(Value::String(type_str)) => {
                match type_str.as_str() {
                    "string" => self.validate_string(schema, data, path),
                    "number" => self.validate_number(schema, data, path),
                    "boolean" => self.validate_boolean(data, path),
                    "object" => {
                        if let Some(properties) = schema.get("properties") {
                            if properties.is_object() {
                                self.validate_object(properties.as_object().unwrap(), data, path)
                            } else {
                                Ok(())
                            }
                        } else {
                            Ok(())
                        }
                    }
                    "array" => self.validate_array(schema, data, path),
                    _ => Ok(()),
                }
            }
            _ => Ok(()),
        }
    }
    
    fn validate_string(
        &self,
        schema: &Value,
        data: &Value,
        path: &str,
    ) -> Result<(), ValidationError> {
        if !data.is_string() {
            return Err(ValidationError {
                message: "Expected string".to_string(),
                path: path.to_string(),
            });
        }
        
        let s = data.as_str().unwrap();
        
        if let Some(Value::Number(min)) = schema.get("minLength") {
            if s.len() < min.as_u64().unwrap() as usize {
                return Err(ValidationError {
                    message: format!("String too short (min: {})", min),
                    path: path.to_string(),
                });
            }
        }
        
        if let Some(Value::Number(max)) = schema.get("maxLength") {
            if s.len() > max.as_u64().unwrap() as usize {
                return Err(ValidationError {
                    message: format!("String too long (max: {})", max),
                    path: path.to_string(),
                });
            }
        }
        
        Ok(())
    }
    
    fn validate_number(
        &self,
        schema: &Value,
        data: &Value,
        path: &str,
    ) -> Result<(), ValidationError> {
        if !data.is_number() {
            return Err(ValidationError {
                message: "Expected number".to_string(),
                path: path.to_string(),
            });
        }
        
        let n = data.as_f64().unwrap();
        
        if let Some(Value::Number(min)) = schema.get("minimum") {
            if n < min.as_f64().unwrap() {
                return Err(ValidationError {
                    message: format!("Number too small (min: {})", min),
                    path: path.to_string(),
                });
            }
        }
        
        if let Some(Value::Number(max)) = schema.get("maximum") {
            if n > max.as_f64().unwrap() {
                return Err(ValidationError {
                    message: format!("Number too large (max: {})", max),
                    path: path.to_string(),
                });
            }
        }
        
        Ok(())
    }
    
    fn validate_boolean(&self, data: &Value, path: &str) -> Result<(), ValidationError> {
        if !data.is_boolean() {
            return Err(ValidationError {
                message: "Expected boolean".to_string(),
                path: path.to_string(),
            });
        }
        Ok(())
    }
    
    fn validate_array(
        &self,
        schema: &Value,
        data: &Value,
        path: &str,
    ) -> Result<(), ValidationError> {
        if !data.is_array() {
            return Err(ValidationError {
                message: "Expected array".to_string(),
                path: path.to_string(),
            });
        }
        
        let arr = data.as_array().unwrap();
        
        if let Some(Value::Number(min)) = schema.get("minItems") {
            if arr.len() < min.as_u64().unwrap() as usize {
                return Err(ValidationError {
                    message: format!("Array too short (min: {})", min),
                    path: path.to_string(),
                });
            }
        }
        
        if let Some(Value::Number(max)) = schema.get("maxItems") {
            if arr.len() > max.as_u64().unwrap() as usize {
                return Err(ValidationError {
                    message: format!("Array too long (max: {})", max),
                    path: path.to_string(),
                });
            }
        }
        
        if let Some(item_schema) = schema.get("items") {
            for (i, item) in arr.iter().enumerate() {
                let item_path = format!("{}[{}]", path, i);
                self.validate_value(item_schema, item, &item_path)?;
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    
    #[test]
    fn test_basic_validation() {
        let schema = json!({
            "type": "object",
            "properties": {
                "name": {
                    "type": "string",
                    "minLength": 1,
                    "maxLength": 50
                },
                "age": {
                    "type": "number",
                    "minimum": 0,
                    "maximum": 150
                },
                "active": {
                    "type": "boolean"
                }
            }
        });
        
        let validator = JsonValidator::new(schema).unwrap();
        
        let valid_data = json!({
            "name": "John Doe",
            "age": 30,
            "active": true
        });
        
        assert!(validator.validate(&valid_data).is_ok());
        
        let invalid_data = json!({
            "name": "",
            "age": 200,
            "active": "yes"
        });
        
        let result = validator.validate(&invalid_data);
        assert!(result.is_err());
    }
}