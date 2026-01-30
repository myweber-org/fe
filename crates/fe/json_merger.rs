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
use serde_json::{Map, Value};
use std::fs;
use std::path::Path;

pub fn merge_json_files(file_paths: &[&str]) -> Result<Value, Box<dyn std::error::Error>> {
    let mut merged_map = Map::new();

    for path_str in file_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            return Err(format!("File not found: {}", path_str).into());
        }

        let content = fs::read_to_string(path)?;
        let json_value: Value = serde_json::from_str(&content)?;

        if let Value::Object(obj) = json_value {
            for (key, value) in obj {
                if merged_map.contains_key(&key) {
                    eprintln!("Warning: Key '{}' already exists, overwriting.", key);
                }
                merged_map.insert(key, value);
            }
        } else {
            return Err("Top-level JSON must be an object".into());
        }
    }

    Ok(Value::Object(merged_map))
}
use serde_json::{Value, Map};
use std::collections::HashSet;
use std::fs;

pub fn merge_json_objects(base: &Value, new: &Value, strategy: MergeStrategy) -> Value {
    match (base, new) {
        (Value::Object(base_map), Value::Object(new_map)) => {
            let mut result = Map::new();
            let all_keys: HashSet<_> = base_map.keys().chain(new_map.keys()).collect();

            for key in all_keys {
                let base_val = base_map.get(key);
                let new_val = new_map.get(key);

                match (base_val, new_val) {
                    (Some(b), Some(n)) => {
                        let merged = merge_json_objects(b, n, strategy.clone());
                        result.insert(key.clone(), merged);
                    }
                    (Some(b), None) => {
                        result.insert(key.clone(), b.clone());
                    }
                    (None, Some(n)) => {
                        result.insert(key.clone(), n.clone());
                    }
                    (None, None) => unreachable!(),
                }
            }
            Value::Object(result)
        }
        (Value::Array(base_arr), Value::Array(new_arr)) => {
            match strategy {
                MergeStrategy::PreferNew => Value::Array(new_arr.clone()),
                MergeStrategy::PreferOld => Value::Array(base_arr.clone()),
                MergeStrategy::Concatenate => {
                    let mut combined = base_arr.clone();
                    combined.extend(new_arr.clone());
                    Value::Array(combined)
                }
            }
        }
        _ => new.clone(),
    }
}

#[derive(Clone)]
pub enum MergeStrategy {
    PreferNew,
    PreferOld,
    Concatenate,
}

pub fn merge_json_files(path1: &str, path2: &str, output_path: &str, strategy: MergeStrategy) -> Result<(), Box<dyn std::error::Error>> {
    let content1 = fs::read_to_string(path1)?;
    let content2 = fs::read_to_string(path2)?;
    
    let json1: Value = serde_json::from_str(&content1)?;
    let json2: Value = serde_json::from_str(&content2)?;
    
    let merged = merge_json_objects(&json1, &json2, strategy);
    let output = serde_json::to_string_pretty(&merged)?;
    
    fs::write(output_path, output)?;
    Ok(())
}