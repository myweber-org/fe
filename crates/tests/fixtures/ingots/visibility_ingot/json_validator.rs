use serde_json::{Value, Map};
use std::collections::HashSet;

pub fn validate_json_schema(data: &Value, schema: &Map<String, Value>) -> Result<(), String> {
    let mut errors = Vec::new();
    validate_object(data, schema, &mut errors, "");
    
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("\n"))
    }
}

fn validate_object(data: &Value, schema: &Map<String, Value>, errors: &mut Vec<String>, path: &str) {
    if let Some(required_fields) = schema.get("required").and_then(|v| v.as_array()) {
        let data_map = data.as_object().unwrap_or(&Map::new());
        let present_keys: HashSet<_> = data_map.keys().collect();
        
        for field in required_fields {
            if let Some(field_name) = field.as_str() {
                if !present_keys.contains(&field_name) {
                    errors.push(format!("{}: Missing required field '{}'", path, field_name));
                }
            }
        }
    }
    
    if let Some(properties) = schema.get("properties").and_then(|v| v.as_object()) {
        for (key, prop_schema) in properties {
            let new_path = if path.is_empty() {
                key.clone()
            } else {
                format!("{}.{}", path, key)
            };
            
            if let Some(data_value) = data.get(key) {
                validate_type(data_value, prop_schema, errors, &new_path);
            }
        }
    }
}

fn validate_type(data: &Value, schema: &Value, errors: &mut Vec<String>, path: &str) {
    if let Some(type_str) = schema.get("type").and_then(|v| v.as_str()) {
        match type_str {
            "string" => {
                if !data.is_string() {
                    errors.push(format!("{}: Expected string, got {:?}", path, data));
                }
            }
            "number" => {
                if !data.is_number() {
                    errors.push(format!("{}: Expected number, got {:?}", path, data));
                }
            }
            "boolean" => {
                if !data.is_boolean() {
                    errors.push(format!("{}: Expected boolean, got {:?}", path, data));
                }
            }
            "object" => {
                if let Some(obj_schema) = schema.as_object() {
                    validate_object(data, obj_schema, errors, path);
                }
            }
            "array" => {
                if let Some(items_schema) = schema.get("items") {
                    if let Some(arr) = data.as_array() {
                        for (i, item) in arr.iter().enumerate() {
                            validate_type(item, items_schema, errors, &format!("{}[{}]", path, i));
                        }
                    } else {
                        errors.push(format!("{}: Expected array, got {:?}", path, data));
                    }
                }
            }
            _ => {}
        }
    }
}