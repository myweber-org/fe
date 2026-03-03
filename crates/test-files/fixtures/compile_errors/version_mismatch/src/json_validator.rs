use serde_json::Value;
use std::fs;

pub fn validate_json_schema(file_path: &str, schema: &Value) -> Result<(), String> {
    let content = fs::read_to_string(file_path)
        .map_err(|e| format!("Failed to read file {}: {}", file_path, e))?;
    
    let data: Value = serde_json::from_str(&content)
        .map_err(|e| format!("Invalid JSON in {}: {}", file_path, e))?;
    
    if !json_schema_validator(&data, schema) {
        return Err(format!("JSON in {} does not match required schema", file_path));
    }
    
    Ok(())
}

fn json_schema_validator(data: &Value, schema: &Value) -> bool {
    match schema.get("type") {
        Some(Value::String(type_str)) => {
            match type_str.as_str() {
                "object" => validate_object(data, schema),
                "array" => validate_array(data, schema),
                "string" => data.is_string(),
                "number" => data.is_number(),
                "boolean" => data.is_boolean(),
                "null" => data.is_null(),
                _ => false,
            }
        }
        _ => false,
    }
}

fn validate_object(data: &Value, schema: &Value) -> bool {
    if !data.is_object() {
        return false;
    }
    
    let obj = data.as_object().unwrap();
    let required = schema.get("required")
        .and_then(|v| v.as_array())
        .unwrap_or(&vec![]);
    
    for field in required {
        if let Value::String(field_name) = field {
            if !obj.contains_key(field_name) {
                return false;
            }
        }
    }
    
    if let Some(properties) = schema.get("properties") {
        if let Value::Object(props) = properties {
            for (key, prop_schema) in props {
                if let Some(value) = obj.get(key) {
                    if !json_schema_validator(value, prop_schema) {
                        return false;
                    }
                }
            }
        }
    }
    
    true
}

fn validate_array(data: &Value, schema: &Value) -> bool {
    if !data.is_array() {
        return false;
    }
    
    let arr = data.as_array().unwrap();
    
    if let Some(Value::Number(min)) = schema.get("minItems") {
        if arr.len() < min.as_u64().unwrap() as usize {
            return false;
        }
    }
    
    if let Some(item_schema) = schema.get("items") {
        for item in arr {
            if !json_schema_validator(item, item_schema) {
                return false;
            }
        }
    }
    
    true
}