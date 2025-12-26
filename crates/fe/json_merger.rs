use serde_json::{Map, Value};
use std::fs;
use std::path::Path;

pub fn merge_json_files(file_paths: &[&str]) -> Result<Value, Box<dyn std::error::Error>> {
    let mut merged_map = Map::new();

    for path_str in file_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            continue;
        }

        let content = fs::read_to_string(path)?;
        let json_value: Value = serde_json::from_str(&content)?;

        if let Value::Object(map) = json_value {
            for (key, value) in map {
                merged_map.insert(key, value);
            }
        }
    }

    Ok(Value::Object(merged_map))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_files() {
        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();

        writeln!(file1, r#"{"name": "Alice", "age": 30}"#).unwrap();
        writeln!(file2, r#"{"city": "Berlin", "active": true}"#).unwrap();

        let result = merge_json_files(&[
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ])
        .unwrap();

        assert_eq!(result["name"], "Alice");
        assert_eq!(result["age"], 30);
        assert_eq!(result["city"], "Berlin");
        assert_eq!(result["active"], true);
    }
}
use std::collections::HashMap;
use serde_json::{Value, Map};

pub fn merge_json(base: &mut Value, update: &Value, resolve_conflict: fn(&str, &Value, &Value) -> Value) -> Value {
    match (base, update) {
        (Value::Object(base_map), Value::Object(update_map)) => {
            for (key, update_val) in update_map {
                if base_map.contains_key(key) {
                    let base_val = base_map.get_mut(key).unwrap();
                    *base_val = merge_json(base_val, update_val, resolve_conflict);
                } else {
                    base_map.insert(key.clone(), update_val.clone());
                }
            }
            Value::Object(std::mem::take(base_map))
        }
        (Value::Array(base_arr), Value::Array(update_arr)) => {
            let mut merged = base_arr.clone();
            merged.extend_from_slice(update_arr);
            Value::Array(merged)
        }
        (base_val, update_val) if base_val != update_val => {
            resolve_conflict("conflict", base_val, update_val)
        }
        _ => base.clone(),
    }
}

pub fn default_resolver(path: &str, base: &Value, update: &Value) -> Value {
    eprintln!("Conflict at {}: base={:?}, update={:?}", path, base, update);
    update.clone()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_merge_objects() {
        let mut base = json!({"a": 1, "b": {"x": 10}});
        let update = json!({"b": {"y": 20}, "c": 3});
        let result = merge_json(&mut base, &update, default_resolver);
        assert_eq!(result, json!({"a": 1, "b": {"x": 10, "y": 20}, "c": 3}));
    }

    #[test]
    fn test_merge_arrays() {
        let mut base = json!([1, 2]);
        let update = json!([3, 4]);
        let result = merge_json(&mut base, &update, default_resolver);
        assert_eq!(result, json!([1, 2, 3, 4]));
    }
}