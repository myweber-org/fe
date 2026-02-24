use serde_json::{Map, Value};
use std::fs;
use std::path::Path;

pub fn merge_json_files(file_paths: &[&str]) -> Result<Value, Box<dyn std::error::Error>> {
    let mut merged_map = Map::new();

    for file_path in file_paths {
        let path = Path::new(file_path);
        if !path.exists() {
            return Err(format!("File not found: {}", file_path).into());
        }

        let content = fs::read_to_string(path)?;
        let json_value: Value = serde_json::from_str(&content)?;

        if let Value::Object(map) = json_value {
            for (key, value) in map {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_files() {
        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();

        writeln!(file1, r#"{"a": 1, "b": "test"}"#).unwrap();
        writeln!(file2, r#"{"c": true, "d": [1,2,3]}"#).unwrap();

        let result = merge_json_files(&[
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ]).unwrap();

        assert_eq!(result["a"], 1);
        assert_eq!(result["b"], "test");
        assert_eq!(result["c"], true);
        assert!(result["d"].is_array());
    }

    #[test]
    fn test_overwrite_keys() {
        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();

        writeln!(file1, r#"{"key": "first"}"#).unwrap();
        writeln!(file2, r#"{"key": "second"}"#).unwrap();

        let result = merge_json_files(&[
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ]).unwrap();

        assert_eq!(result["key"], "second");
    }
}
use serde_json::{Value, Map};
use std::fs;
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P], output_path: P) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged = Map::new();
    
    for path in paths {
        let content = fs::read_to_string(path)?;
        let json: Value = serde_json::from_str(&content)?;
        
        if let Value::Object(obj) = json {
            merge_objects(&mut merged, obj);
        }
    }
    
    let output_json = Value::Object(merged);
    let output_str = serde_json::to_string_pretty(&output_json)?;
    fs::write(output_path, output_str)?;
    
    Ok(())
}

fn merge_objects(target: &mut Map<String, Value>, source: Map<String, Value>) {
    for (key, value) in source {
        if !target.contains_key(&key) {
            target.insert(key.clone(), value);
        } else {
            let existing = target.get_mut(&key).unwrap();
            match (existing, value) {
                (Value::Object(ref mut target_obj), Value::Object(source_obj)) => {
                    merge_objects(target_obj, source_obj);
                }
                (Value::Array(ref mut target_arr), Value::Array(source_arr)) => {
                    target_arr.extend(source_arr);
                    target_arr.sort();
                    target_arr.dedup();
                }
                (_, new_value) => {
                    *existing = new_value;
                }
            }
        }
    }
}
use serde_json::{Value, Map};
use std::fs;
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P], output_path: P) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged: Map<String, Value> = Map::new();

    for path in paths {
        let content = fs::read_to_string(path)?;
        let json: Value = serde_json::from_str(&content)?;

        if let Value::Object(obj) = json {
            for (key, value) in obj {
                if merged.contains_key(&key) {
                    let existing = &merged[&key];
                    if existing != &value {
                        let resolved = resolve_conflict(&key, existing, &value);
                        merged.insert(key, resolved);
                    }
                } else {
                    merged.insert(key, value);
                }
            }
        }
    }

    let output_json = Value::Object(merged);
    let serialized = serde_json::to_string_pretty(&output_json)?;
    fs::write(output_path, serialized)?;

    Ok(())
}

fn resolve_conflict(key: &str, existing: &Value, new: &Value) -> Value {
    match (existing, new) {
        (Value::Array(arr1), Value::Array(arr2)) => {
            let mut combined = arr1.clone();
            combined.extend(arr2.clone());
            Value::Array(combined)
        },
        (Value::Object(obj1), Value::Object(obj2)) => {
            let mut merged_obj = obj1.clone();
            for (k, v) in obj2 {
                if merged_obj.contains_key(k) {
                    let resolved = resolve_conflict(k, &merged_obj[k], v);
                    merged_obj.insert(k.clone(), resolved);
                } else {
                    merged_obj.insert(k.clone(), v.clone());
                }
            }
            Value::Object(merged_obj)
        },
        _ => {
            eprintln!("Conflict detected for key '{}', using new value", key);
            new.clone()
        }
    }
}