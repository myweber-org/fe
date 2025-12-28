use serde_json::{Map, Value};
use std::collections::HashSet;

pub fn merge_json(base: &mut Value, update: &Value, overwrite_arrays: bool) {
    match (base, update) {
        (Value::Object(base_map), Value::Object(update_map)) => {
            for (key, update_value) in update_map {
                if let Some(base_value) = base_map.get_mut(key) {
                    merge_json(base_value, update_value, overwrite_arrays);
                } else {
                    base_map.insert(key.clone(), update_value.clone());
                }
            }
        }
        (Value::Array(base_arr), Value::Array(update_arr)) if !overwrite_arrays => {
            let mut existing_set = HashSet::new();
            for item in base_arr.iter() {
                if let Some(obj) = item.as_object() {
                    if let Some(id) = obj.get("id").and_then(|v| v.as_str()) {
                        existing_set.insert(id.to_string());
                    }
                }
            }
            
            for item in update_arr {
                if let Some(obj) = item.as_object() {
                    if let Some(id) = obj.get("id").and_then(|v| v.as_str()) {
                        if !existing_set.contains(id) {
                            base_arr.push(item.clone());
                        }
                    } else {
                        base_arr.push(item.clone());
                    }
                } else {
                    base_arr.push(item.clone());
                }
            }
        }
        (base, update) => {
            *base = update.clone();
        }
    }
}

pub fn merge_json_with_strategy(
    base: &str,
    update: &str,
    overwrite_arrays: bool,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut base_value: Value = serde_json::from_str(base)?;
    let update_value: Value = serde_json::from_str(update)?;
    
    merge_json(&mut base_value, &update_value, overwrite_arrays);
    
    Ok(serde_json::to_string_pretty(&base_value)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_merge() {
        let base = r#"{"name": "Alice", "age": 30}"#;
        let update = r#"{"age": 31, "city": "New York"}"#;
        
        let result = merge_json_with_strategy(base, update, false).unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        
        assert_eq!(parsed["name"], "Alice");
        assert_eq!(parsed["age"], 31);
        assert_eq!(parsed["city"], "New York");
    }
    
    #[test]
    fn test_array_merge_with_ids() {
        let base = r#"{"items": [{"id": "1", "name": "first"}, {"id": "2", "name": "second"}]}"#;
        let update = r#"{"items": [{"id": "2", "name": "updated"}, {"id": "3", "name": "third"}]}"#;
        
        let result = merge_json_with_strategy(base, update, false).unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        
        let items = parsed["items"].as_array().unwrap();
        assert_eq!(items.len(), 3);
    }
}