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

        if let Value::Object(map) = json_value {
            for (key, value) in map {
                if merged_map.contains_key(&key) {
                    eprintln!("Warning: Key '{}' already exists, overwriting.", key);
                }
                merged_map.insert(key, value);
            }
        } else {
            return Err("Each JSON file must contain a JSON object at the top level.".into());
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

        let paths = [
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ];

        let result = merge_json_files(&paths).unwrap();
        let obj = result.as_object().unwrap();

        assert_eq!(obj.get("a").unwrap().as_i64(), Some(1));
        assert_eq!(obj.get("b").unwrap().as_str(), Some("test"));
        assert_eq!(obj.get("c").unwrap().as_bool(), Some(true));
        assert!(obj.get("d").unwrap().is_array());
    }

    #[test]
    fn test_overwrite_key() {
        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();

        writeln!(file1, r#"{"key": "first"}"#).unwrap();
        writeln!(file2, r#"{"key": "second"}"#).unwrap();

        let paths = [
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ];

        let result = merge_json_files(&paths).unwrap();
        let obj = result.as_object().unwrap();

        assert_eq!(obj.get("key").unwrap().as_str(), Some("second"));
    }
}
use serde_json::{Map, Value};
use std::collections::HashSet;
use std::fs;
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P], output_path: P) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged = Map::new();
    let mut processed_keys = HashSet::new();
    let mut conflict_log = Vec::new();

    for path in paths {
        let content = fs::read_to_string(path)?;
        let json: Value = serde_json::from_str(&content)?;

        if let Value::Object(obj) = json {
            for (key, value) in obj {
                if processed_keys.contains(&key) {
                    conflict_log.push(format!("Conflict detected for key '{}'", key));
                    if let Some(existing) = merged.get_mut(&key) {
                        *existing = merge_values(existing.clone(), value);
                    }
                } else {
                    merged.insert(key.clone(), value);
                    processed_keys.insert(key);
                }
            }
        }
    }

    let output_json = Value::Object(merged);
    let pretty_json = serde_json::to_string_pretty(&output_json)?;
    fs::write(output_path, pretty_json)?;

    if !conflict_log.is_empty() {
        eprintln!("Merged with conflicts:");
        for log in conflict_log {
            eprintln!("  {}", log);
        }
    }

    Ok(())
}

fn merge_values(a: Value, b: Value) -> Value {
    match (a, b) {
        (Value::Array(mut arr_a), Value::Array(arr_b)) => {
            arr_a.extend(arr_b);
            Value::Array(arr_a)
        }
        (Value::Object(mut obj_a), Value::Object(obj_b)) => {
            for (key, val_b) in obj_b {
                if let Some(val_a) = obj_a.get_mut(&key) {
                    *val_a = merge_values(val_a.clone(), val_b);
                } else {
                    obj_a.insert(key, val_b);
                }
            }
            Value::Object(obj_a)
        }
        (_, b_val) => b_val,
    }
}
use serde_json::{Map, Value};
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(input_paths: &[P], output_path: P) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged_map = Map::new();

    for input_path in input_paths {
        let file = File::open(input_path)?;
        let reader = BufReader::new(file);
        let json_value: Value = serde_json::from_reader(reader)?;

        if let Value::Object(map) = json_value {
            for (key, value) in map {
                if merged_map.contains_key(&key) {
                    eprintln!("Warning: Duplicate key '{}' found. Overwriting.", key);
                }
                merged_map.insert(key, value);
            }
        } else {
            return Err("Input JSON is not an object".into());
        }
    }

    let merged_value = Value::Object(merged_map);
    let mut output_file = File::create(output_path)?;
    write!(output_file, "{}", serde_json::to_string_pretty(&merged_value)?)?;

    Ok(())
}use serde_json::{Map, Value};
use std::fs;
use std::path::Path;

pub fn merge_json_files(input_paths: &[&str], output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged_map = Map::new();

    for path_str in input_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            return Err(format!("File not found: {}", path_str).into());
        }

        let content = fs::read_to_string(path)?;
        let json_value: Value = serde_json::from_str(&content)?;

        if let Value::Object(map) = json_value {
            for (key, value) in map {
                merged_map.insert(key, value);
            }
        } else {
            return Err("Input JSON is not an object".into());
        }
    }

    let merged_value = Value::Object(merged_map);
    let output_json = serde_json::to_string_pretty(&merged_value)?;
    fs::write(output_path, output_json)?;

    Ok(())
}use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, Read};
use std::path::Path;

type JsonValue = serde_json::Value;
type JsonMap = HashMap<String, JsonValue>;

pub fn merge_json_files(file_paths: &[&str]) -> Result<JsonValue, Box<dyn std::error::Error>> {
    let mut merged_map = JsonMap::new();

    for path_str in file_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            eprintln!("Warning: File {} not found, skipping.", path_str);
            continue;
        }

        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        let mut content = String::new();
        reader.read_to_string(&mut content)?;

        let json_value: JsonValue = serde_json::from_str(&content)?;

        if let JsonValue::Object(map) = json_value {
            for (key, value) in map {
                if merged_map.contains_key(&key) {
                    eprintln!("Warning: Key '{}' already exists, overwriting.", key);
                }
                merged_map.insert(key, value);
            }
        } else {
            return Err("Top-level JSON value must be an object".into());
        }
    }

    Ok(JsonValue::Object(
        merged_map
            .into_iter()
            .map(|(k, v)| (k, v))
            .collect::<serde_json::Map<String, JsonValue>>(),
    ))
}

pub fn write_merged_json(
    output_path: &str,
    json_value: &JsonValue,
) -> Result<(), Box<dyn std::error::Error>> {
    let json_string = serde_json::to_string_pretty(json_value)?;
    fs::write(output_path, json_string)?;
    println!("Merged JSON written to {}", output_path);
    Ok(())
}use serde_json::{json, Value};
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

pub fn merge_json_files(file_paths: &[&str], deduplicate_by_key: Option<&str>) -> Result<Value, String> {
    let mut merged_array = Vec::new();
    let mut seen_keys = HashSet::new();

    for path_str in file_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            return Err(format!("File not found: {}", path_str));
        }

        let mut file = File::open(path).map_err(|e| e.to_string())?;
        let mut contents = String::new();
        file.read_to_string(&mut contents).map_err(|e| e.to_string())?;

        let json_value: Value = serde_json::from_str(&contents).map_err(|e| e.to_string())?;

        match json_value {
            Value::Array(arr) => {
                for item in arr {
                    if let Some(key) = deduplicate_by_key {
                        if let Some(obj) = item.as_object() {
                            if let Some(key_value) = obj.get(key) {
                                let key_str = key_value.to_string();
                                if !seen_keys.contains(&key_str) {
                                    seen_keys.insert(key_str);
                                    merged_array.push(item);
                                }
                                continue;
                            }
                        }
                    }
                    merged_array.push(item);
                }
            }
            Value::Object(_) => {
                if let Some(key) = deduplicate_by_key {
                    if let Some(obj) = json_value.as_object() {
                        if let Some(key_value) = obj.get(key) {
                            let key_str = key_value.to_string();
                            if !seen_keys.contains(&key_str) {
                                seen_keys.insert(key_str);
                                merged_array.push(json_value);
                            }
                        } else {
                            merged_array.push(json_value);
                        }
                    }
                } else {
                    merged_array.push(json_value);
                }
            }
            _ => return Err("JSON root must be an array or object".to_string()),
        }
    }

    Ok(Value::Array(merged_array))
}

pub fn write_merged_json(output_path: &str, value: &Value) -> Result<(), String> {
    let file = File::create(output_path).map_err(|e| e.to_string())?;
    serde_json::to_writer_pretty(file, value).map_err(|e| e.to_string())?;
    Ok(())
}