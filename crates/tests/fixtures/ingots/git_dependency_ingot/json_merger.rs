use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, Read};
use std::path::Path;

type JsonValue = serde_json::Value;

pub fn merge_json_files(file_paths: &[impl AsRef<Path>]) -> Result<JsonValue, Box<dyn std::error::Error>> {
    let mut merged_array = Vec::new();

    for path in file_paths {
        let file = File::open(path.as_ref())?;
        let mut reader = BufReader::new(file);
        let mut contents = String::new();
        reader.read_to_string(&mut contents)?;

        let json_value: JsonValue = serde_json::from_str(&contents)?;
        
        match json_value {
            JsonValue::Array(arr) => {
                merged_array.extend(arr);
            }
            _ => {
                merged_array.push(json_value);
            }
        }
    }

    Ok(JsonValue::Array(merged_array))
}

pub fn merge_json_with_key(
    file_paths: &[impl AsRef<Path>],
    key: &str,
) -> Result<JsonValue, Box<dyn std::error::Error>> {
    let mut merged_map = HashMap::new();

    for path in file_paths {
        let contents = fs::read_to_string(path.as_ref())?;
        let json_value: JsonValue = serde_json::from_str(&contents)?;

        if let JsonValue::Object(obj) = json_value {
            if let Some(value) = obj.get(key) {
                merged_map.insert(key.to_string(), value.clone());
            }
        }
    }

    Ok(JsonValue::Object(serde_json::Map::from_iter(merged_map)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_files() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();

        fs::write(&file1, r#"[{"id": 1}, {"id": 2}]"#).unwrap();
        fs::write(&file2, r#"{"id": 3}"#).unwrap();

        let result = merge_json_files(&[file1.path(), file2.path()]).unwrap();
        assert!(result.is_array());
        assert_eq!(result.as_array().unwrap().len(), 3);
    }

    #[test]
    fn test_merge_json_with_key() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();

        fs::write(&file1, r#"{"user": "alice", "age": 30}"#).unwrap();
        fs::write(&file2, r#"{"user": "bob", "age": 25}"#).unwrap();

        let result = merge_json_with_key(&[file1.path(), file2.path()], "user").unwrap();
        assert!(result.is_object());
        let obj = result.as_object().unwrap();
        assert!(obj.contains_key("user"));
    }
}
use serde_json::{Map, Value};
use std::env;
use std::fs;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <output_file.json> <input1.json> [input2.json ...]", args[0]);
        process::exit(1);
    }

    let output_path = &args[1];
    let input_paths = &args[2..];

    let mut merged_map = Map::new();

    for input_path in input_paths {
        let content = match fs::read_to_string(input_path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Failed to read {}: {}", input_path, e);
                process::exit(1);
            }
        };

        let json_value: Value = match serde_json::from_str(&content) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("Failed to parse JSON from {}: {}", input_path, e);
                process::exit(1);
            }
        };

        if let Value::Object(map) = json_value {
            for (key, value) in map {
                merged_map.insert(key, value);
            }
        } else {
            eprintln!("Top-level JSON in {} is not an object", input_path);
            process::exit(1);
        }
    }

    let merged_value = Value::Object(merged_map);
    let json_string = match serde_json::to_string_pretty(&merged_value) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to serialize merged JSON: {}", e);
            process::exit(1);
        }
    };

    if let Err(e) = fs::write(output_path, json_string) {
        eprintln!("Failed to write to {}: {}", output_path, e);
        process::exit(1);
    }

    println!("Successfully merged {} files into {}", input_paths.len(), output_path);
}use serde_json::{json, Value};
use std::collections::HashSet;
use std::fs;
use std::path::Path;

pub fn merge_json_files(file_paths: &[&str], output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged_array = Vec::new();
    let mut seen_ids = HashSet::new();

    for file_path in file_paths {
        let content = fs::read_to_string(file_path)?;
        let json_value: Value = serde_json::from_str(&content)?;

        if let Value::Array(arr) = json_value {
            for item in arr {
                if let Some(id) = item.get("id").and_then(|v| v.as_str()) {
                    if !seen_ids.contains(id) {
                        seen_ids.insert(id.to_string());
                        merged_array.push(item);
                    }
                } else {
                    merged_array.push(item);
                }
            }
        } else {
            merged_array.push(json_value);
        }
    }

    let output_json = json!(merged_array);
    fs::write(output_path, output_json.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_files() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();

        fs::write(file1.path(), r#"[{"id": "1", "name": "Alice"}, {"id": "2", "name": "Bob"}]"#).unwrap();
        fs::write(file2.path(), r#"[{"id": "2", "name": "Bob"}, {"id": "3", "name": "Charlie"}]"#).unwrap();

        let paths = vec![file1.path().to_str().unwrap(), file2.path().to_str().unwrap()];
        merge_json_files(&paths, output_file.path().to_str().unwrap()).unwrap();

        let content = fs::read_to_string(output_file.path()).unwrap();
        let parsed: Value = serde_json::from_str(&content).unwrap();
        assert_eq!(parsed.as_array().unwrap().len(), 3);
    }
}use serde_json::{Map, Value};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub fn merge_json_files(file_paths: &[&str]) -> Result<Value, Box<dyn std::error::Error>> {
    let mut merged_map = Map::new();

    for &file_path in file_paths {
        if !Path::new(file_path).exists() {
            eprintln!("Warning: File {} not found, skipping.", file_path);
            continue;
        }

        let content = fs::read_to_string(file_path)?;
        let json_value: Value = serde_json::from_str(&content)?;

        if let Value::Object(map) = json_value {
            for (key, value) in map {
                if merged_map.contains_key(&key) {
                    eprintln!("Warning: Key '{}' already exists, overwriting.", key);
                }
                merged_map.insert(key, value);
            }
        } else {
            return Err("Root of JSON file is not an object".into());
        }
    }

    Ok(Value::Object(merged_map))
}

pub fn merge_json_with_strategy(
    file_paths: &[&str],
    strategy: MergeStrategy,
) -> Result<Value, Box<dyn std::error::Error>> {
    let mut accumulator: HashMap<String, Vec<Value>> = HashMap::new();

    for &file_path in file_paths {
        if !Path::new(file_path).exists() {
            eprintln!("Warning: File {} not found, skipping.", file_path);
            continue;
        }

        let content = fs::read_to_string(file_path)?;
        let json_value: Value = serde_json::from_str(&content)?;

        if let Value::Object(map) = json_value {
            for (key, value) in map {
                accumulator.entry(key).or_default().push(value);
            }
        } else {
            return Err("Root of JSON file is not an object".into());
        }
    }

    let merged_map = match strategy {
        MergeStrategy::Overwrite => {
            let mut map = Map::new();
            for (key, values) in accumulator {
                if let Some(last) = values.last() {
                    map.insert(key, last.clone());
                }
            }
            map
        }
        MergeStrategy::CombineArrays => {
            let mut map = Map::new();
            for (key, values) in accumulator {
                map.insert(key, Value::Array(values));
            }
            map
        }
        MergeStrategy::MergeObjects => {
            let mut map = Map::new();
            for (key, values) in accumulator {
                let mut combined = Map::new();
                for value in values {
                    if let Value::Object(obj) = value {
                        for (sub_key, sub_value) in obj {
                            combined.insert(sub_key, sub_value);
                        }
                    }
                }
                map.insert(key, Value::Object(combined));
            }
            map
        }
    };

    Ok(Value::Object(merged_map))
}

#[derive(Debug, Clone, Copy)]
pub enum MergeStrategy {
    Overwrite,
    CombineArrays,
    MergeObjects,
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
            for (key, value) in obj {
                merge_value(&mut merged, key, value);
            }
        }
    }

    let output_json = Value::Object(merged);
    let serialized = serde_json::to_string_pretty(&output_json)?;
    fs::write(output_path, serialized)?;

    Ok(())
}

fn merge_value(map: &mut Map<String, Value>, key: String, new_value: Value) {
    match map.get_mut(&key) {
        Some(existing_value) => {
            if let (Value::Object(existing_obj), Value::Object(new_obj)) = (existing_value, &new_value) {
                let mut existing_obj = existing_obj.clone();
                for (nested_key, nested_value) in new_obj {
                    merge_value(&mut existing_obj, nested_key.clone(), nested_value.clone());
                }
                map.insert(key, Value::Object(existing_obj));
            } else if existing_value != &new_value {
                let conflict_key = format!("{}_conflict", key);
                map.insert(conflict_key, new_value);
            }
        }
        None => {
            map.insert(key, new_value);
        }
    }
}