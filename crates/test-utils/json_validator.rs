use serde_json::{Value, json};
use std::collections::HashSet;

pub fn validate_json_schema(data: &str, schema: &Value) -> Result<(), String> {
    let parsed_data: Value = serde_json::from_str(data)
        .map_err(|e| format!("Invalid JSON: {}", e))?;

    validate_value(&parsed_data, schema)
}

fn validate_value(data: &Value, schema: &Value) -> Result<(), String> {
    match schema.get("type").and_then(Value::as_str) {
        Some("object") => validate_object(data, schema),
        Some("array") => validate_array(data, schema),
        Some("string") => validate_string(data, schema),
        Some("number") => validate_number(data, schema),
        Some("boolean") => validate_boolean(data, schema),
        Some("null") => validate_null(data),
        Some(t) => Err(format!("Unsupported schema type: {}", t)),
        None => Ok(()),
    }
}

fn validate_object(data: &Value, schema: &Value) -> Result<(), String> {
    if !data.is_object() {
        return Err("Expected object type".to_string());
    }

    let obj = data.as_object().unwrap();
    let required_fields: HashSet<&str> = schema
        .get("required")
        .and_then(Value::as_array)
        .map(|arr| arr.iter().filter_map(Value::as_str).collect())
        .unwrap_or_default();

    for field in &required_fields {
        if !obj.contains_key(*field) {
            return Err(format!("Missing required field: {}", field));
        }
    }

    if let Some(properties) = schema.get("properties").and_then(Value::as_object) {
        for (key, prop_schema) in properties {
            if let Some(value) = obj.get(key) {
                validate_value(value, prop_schema)
                    .map_err(|e| format!("Field '{}': {}", key, e))?;
            }
        }
    }

    Ok(())
}

fn validate_array(data: &Value, schema: &Value) -> Result<(), String> {
    if !data.is_array() {
        return Err("Expected array type".to_string());
    }

    let arr = data.as_array().unwrap();
    
    if let Some(min_items) = schema.get("minItems").and_then(Value::as_u64) {
        if arr.len() < min_items as usize {
            return Err(format!("Array must have at least {} items", min_items));
        }
    }

    if let Some(item_schema) = schema.get("items") {
        for (i, item) in arr.iter().enumerate() {
            validate_value(item, item_schema)
                .map_err(|e| format!("Item {}: {}", i, e))?;
        }
    }

    Ok(())
}

fn validate_string(data: &Value, schema: &Value) -> Result<(), String> {
    if !data.is_string() {
        return Err("Expected string type".to_string());
    }

    let s = data.as_str().unwrap();
    
    if let Some(min_len) = schema.get("minLength").and_then(Value::as_u64) {
        if s.len() < min_len as usize {
            return Err(format!("String must be at least {} characters", min_len));
        }
    }

    if let Some(max_len) = schema.get("maxLength").and_then(Value::as_u64) {
        if s.len() > max_len as usize {
            return Err(format!("String must be at most {} characters", max_len));
        }
    }

    if let Some(pattern) = schema.get("pattern").and_then(Value::as_str) {
        let re = regex::Regex::new(pattern)
            .map_err(|e| format!("Invalid regex pattern: {}", e))?;
        if !re.is_match(s) {
            return Err(format!("String must match pattern: {}", pattern));
        }
    }

    Ok(())
}

fn validate_number(data: &Value, schema: &Value) -> Result<(), String> {
    if !data.is_number() {
        return Err("Expected number type".to_string());
    }

    let num = data.as_f64().unwrap();
    
    if let Some(minimum) = schema.get("minimum").and_then(Value::as_f64) {
        if num < minimum {
            return Err(format!("Number must be at least {}", minimum));
        }
    }

    if let Some(maximum) = schema.get("maximum").and_then(Value::as_f64) {
        if num > maximum {
            return Err(format!("Number must be at most {}", maximum));
        }
    }

    Ok(())
}

fn validate_boolean(data: &Value, _schema: &Value) -> Result<(), String> {
    if !data.is_boolean() {
        return Err("Expected boolean type".to_string());
    }
    Ok(())
}

fn validate_null(data: &Value) -> Result<(), String> {
    if !data.is_null() {
        return Err("Expected null type".to_string());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_validation() {
        let schema = json!({
            "type": "object",
            "required": ["name", "age"],
            "properties": {
                "name": {"type": "string", "minLength": 1},
                "age": {"type": "number", "minimum": 0},
                "email": {"type": "string", "pattern": "^[^@]+@[^@]+\\.[^@]+$"}
            }
        });

        let valid_data = r#"{"name": "John", "age": 30, "email": "john@example.com"}"#;
        assert!(validate_json_schema(valid_data, &schema).is_ok());

        let missing_field = r#"{"name": "John"}"#;
        assert!(validate_json_schema(missing_field, &schema).is_err());

        let invalid_email = r#"{"name": "John", "age": 30, "email": "invalid"}"#;
        assert!(validate_json_schema(invalid_email, &schema).is_err());
    }
}