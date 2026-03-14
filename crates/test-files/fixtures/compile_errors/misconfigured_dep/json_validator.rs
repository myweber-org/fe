use serde_json::{Value, json};
use std::fs;

pub fn validate_json_schema(data: &Value, schema: &Value) -> Result<(), String> {
    if !schema.is_object() {
        return Err("Schema must be a JSON object".to_string());
    }

    if let Some(required_type) = schema.get("type").and_then(|t| t.as_str()) {
        match required_type {
            "string" => {
                if !data.is_string() {
                    return Err(format!("Expected string, got {:?}", data));
                }
            }
            "number" => {
                if !data.is_number() {
                    return Err(format!("Expected number, got {:?}", data));
                }
            }
            "boolean" => {
                if !data.is_bool() {
                    return Err(format!("Expected boolean, got {:?}", data));
                }
            }
            "array" => {
                if !data.is_array() {
                    return Err(format!("Expected array, got {:?}", data));
                }
                if let Some(items_schema) = schema.get("items") {
                    for item in data.as_array().unwrap() {
                        validate_json_schema(item, items_schema)?;
                    }
                }
            }
            "object" => {
                if !data.is_object() {
                    return Err(format!("Expected object, got {:?}", data));
                }
                if let Some(properties) = schema.get("properties").and_then(|p| p.as_object()) {
                    for (key, prop_schema) in properties {
                        if let Some(value) = data.get(key) {
                            validate_json_schema(value, prop_schema)?;
                        } else if let Some(required) = schema.get("required").and_then(|r| r.as_array()) {
                            if required.iter().any(|r| r.as_str() == Some(key)) {
                                return Err(format!("Missing required field: {}", key));
                            }
                        }
                    }
                }
            }
            _ => return Err(format!("Unsupported type: {}", required_type)),
        }
    }

    Ok(())
}

pub fn validate_json_file(file_path: &str, schema: &Value) -> Result<(), String> {
    let content = fs::read_to_string(file_path)
        .map_err(|e| format!("Failed to read file: {}", e))?;
    
    let data: Value = serde_json::from_str(&content)
        .map_err(|e| format!("Invalid JSON: {}", e))?;
    
    validate_json_schema(&data, schema)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_validation() {
        let schema = json!({"type": "string"});
        assert!(validate_json_schema(&json!("hello"), &schema).is_ok());
        assert!(validate_json_schema(&json!(42), &schema).is_err());
    }

    #[test]
    fn test_object_validation() {
        let schema = json!({
            "type": "object",
            "properties": {
                "name": {"type": "string"},
                "age": {"type": "number"}
            },
            "required": ["name"]
        });

        let valid_data = json!({"name": "Alice", "age": 30});
        assert!(validate_json_schema(&valid_data, &schema).is_ok());

        let missing_name = json!({"age": 30});
        assert!(validate_json_schema(&missing_name, &schema).is_err());
    }
}