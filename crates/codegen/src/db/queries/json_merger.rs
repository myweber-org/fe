
use serde_json::{Value, Map};
use std::fs;
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P]) -> Result<Value, Box<dyn std::error::Error>> {
    let mut merged = Map::new();

    for path in paths {
        let content = fs::read_to_string(path)?;
        let json: Value = serde_json::from_str(&content)?;

        if let Value::Object(obj) = json {
            for (key, value) in obj {
                merge_value(&mut merged, key, value);
            }
        }
    }

    Ok(Value::Object(merged))
}

fn merge_value(map: &mut Map<String, Value>, key: String, new_value: Value) {
    match map.get_mut(&key) {
        Some(existing) if existing.is_object() && new_value.is_object() => {
            if let (Value::Object(existing_obj), Value::Object(new_obj)) = (existing, new_value) {
                for (nested_key, nested_value) in new_obj {
                    merge_value(existing_obj, nested_key, nested_value);
                }
            }
        }
        Some(existing) if existing.is_array() && new_value.is_array() => {
            if let (Value::Array(existing_arr), Value::Array(new_arr)) = (existing, new_value) {
                existing_arr.extend(new_arr);
            }
        }
        _ => {
            map.insert(key, new_value);
        }
    }
}