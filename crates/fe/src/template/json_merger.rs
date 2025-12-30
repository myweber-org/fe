
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
        Some(existing) => {
            if let (Value::Object(existing_obj), Value::Object(new_obj)) = (existing, &new_value) {
                let mut existing_map = existing_obj.clone();
                for (k, v) in new_obj {
                    merge_value(&mut existing_map, k.clone(), v.clone());
                }
                *existing = Value::Object(existing_map);
            } else if let (Value::Array(existing_arr), Value::Array(new_arr)) = (existing, &new_value) {
                let mut combined = existing_arr.clone();
                combined.extend(new_arr.clone());
                *existing = Value::Array(combined);
            } else {
                *existing = new_value;
            }
        }
        None => {
            map.insert(key, new_value);
        }
    }
}