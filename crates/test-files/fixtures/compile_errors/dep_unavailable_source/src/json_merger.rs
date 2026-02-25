use serde_json::{Value, Map};
use std::fs;
use std::path::Path;

pub fn merge_json_files(file_paths: &[&str], output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged_map = Map::new();

    for file_path in file_paths {
        let content = fs::read_to_string(file_path)?;
        let json_value: Value = serde_json::from_str(&content)?;

        if let Value::Object(map) = json_value {
            for (key, value) in map {
                merged_map.insert(key, value);
            }
        } else {
            return Err("Each JSON file must contain a JSON object at the top level".into());
        }
    }

    let merged_json = Value::Object(merged_map);
    let json_string = serde_json::to_string_pretty(&merged_json)?;
    fs::write(output_path, json_string)?;

    Ok(())
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
                if merged.contains_key(&key) {
                    let existing = merged.get(&key).unwrap();
                    if existing != &value {
                        let conflict_key = format!("{}_conflict", key);
                        merged.insert(conflict_key, value);
                    }
                } else {
                    merged.insert(key, value);
                }
            }
        }
    }

    let output_json = Value::Object(merged);
    let output_str = serde_json::to_string_pretty(&output_json)?;
    fs::write(output_path, output_str)?;

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
        let output = NamedTempFile::new().unwrap();

        fs::write(&file1, r#"{"a": 1, "b": 2}"#).unwrap();
        fs::write(&file2, r#"{"b": 3, "c": 4}"#).unwrap();

        merge_json_files(&[&file1, &file2], &output).unwrap();

        let content = fs::read_to_string(output).unwrap();
        let parsed: Value = serde_json::from_str(&content).unwrap();

        assert_eq!(parsed["a"], 1);
        assert_eq!(parsed["b"], 2);
        assert_eq!(parsed["b_conflict"], 3);
        assert_eq!(parsed["c"], 4);
    }
}use serde_json::{json, Value};
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P], deduplicate: bool) -> Result<Value, String> {
    let mut merged_array = Vec::new();
    let mut seen_keys = HashSet::new();

    for path in paths {
        let file = File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;
        let mut reader = BufReader::new(file);
        let mut contents = String::new();
        reader
            .read_to_string(&mut contents)
            .map_err(|e| format!("Failed to read file: {}", e))?;

        let value: Value = serde_json::from_str(&contents)
            .map_err(|e| format!("Invalid JSON in file: {}", e))?;

        match value {
            Value::Array(arr) => {
                for item in arr {
                    if deduplicate {
                        if let Some(id) = item.get("id").and_then(|v| v.as_str()) {
                            if seen_keys.insert(id.to_string()) {
                                merged_array.push(item);
                            }
                        } else {
                            merged_array.push(item);
                        }
                    } else {
                        merged_array.push(item);
                    }
                }
            }
            _ => {
                return Err("Each JSON file must contain a JSON array at root level".to_string())
            }
        }
    }

    Ok(Value::Array(merged_array))
}

pub fn write_merged_json<P: AsRef<Path>>(output_path: P, value: &Value) -> Result<(), String> {
    let file = File::create(output_path).map_err(|e| format!("Failed to create output file: {}", e))?;
    serde_json::to_writer_pretty(file, value).map_err(|e| format!("Failed to write JSON: {}", e))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_files() {
        let json1 = r#"[{"id": "1", "name": "Alice"}, {"id": "2", "name": "Bob"}]"#;
        let json2 = r#"[{"id": "3", "name": "Charlie"}, {"id": "1", "name": "Duplicate"}]"#;

        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();
        file1.write_all(json1.as_bytes()).unwrap();
        file2.write_all(json2.as_bytes()).unwrap();

        let paths = [file1.path(), file2.path()];
        let result = merge_json_files(&paths, false).unwrap();
        assert_eq!(result.as_array().unwrap().len(), 4);

        let dedup_result = merge_json_files(&paths, true).unwrap();
        assert_eq!(dedup_result.as_array().unwrap().len(), 3);
    }
}use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub fn merge_json_files(file_paths: &[&str]) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let mut merged_map = HashMap::new();

    for path_str in file_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            return Err(format!("File not found: {}", path_str).into());
        }

        let content = fs::read_to_string(path)?;
        let json_value: serde_json::Value = serde_json::from_str(&content)?;

        if let serde_json::Value::Object(map) = json_value {
            for (key, value) in map {
                merged_map.insert(key, value);
            }
        } else {
            return Err("Each JSON file must contain a JSON object".into());
        }
    }

    Ok(serde_json::Value::Object(merged_map))
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
        ]).unwrap();

        assert_eq!(result["name"], "Alice");
        assert_eq!(result["age"], 30);
        assert_eq!(result["city"], "Berlin");
        assert_eq!(result["active"], true);
    }

    #[test]
    fn test_merge_with_duplicate_keys() {
        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();

        writeln!(file1, r#"{"id": 1, "value": "first"}"#).unwrap();
        writeln!(file2, r#"{"id": 2, "extra": "data"}"#).unwrap();

        let result = merge_json_files(&[
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ]).unwrap();

        assert_eq!(result["id"], 2);
        assert_eq!(result["value"], "first");
        assert_eq!(result["extra"], "data");
    }
}use serde_json::{Map, Value};
use std::fs;
use std::path::Path;

pub fn merge_json_files(file_paths: &[&str]) -> Result<Value, Box<dyn std::error::Error>> {
    let mut merged_map = Map::new();

    for path_str in file_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            eprintln!("Warning: File {} not found, skipping.", path_str);
            continue;
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
            eprintln!("Warning: File {} does not contain a JSON object, skipping.", path_str);
        }
    }

    Ok(Value::Object(merged_map))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_files() {
        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();

        writeln!(file1, r#"{"name": "Alice", "age": 30}"#).unwrap();
        writeln!(file2, r#"{"city": "Berlin", "active": true}"#).unwrap();

        let paths = [
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ];

        let result = merge_json_files(&paths).unwrap();
        let expected = json!({
            "name": "Alice",
            "age": 30,
            "city": "Berlin",
            "active": true
        });

        assert_eq!(result, expected);
    }

    #[test]
    fn test_merge_with_overwrite() {
        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();

        writeln!(file1, r#"{"id": 1, "value": "old"}"#).unwrap();
        writeln!(file2, r#"{"id": 2, "value": "new"}"#).unwrap();

        let paths = [
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ];

        let result = merge_json_files(&paths).unwrap();
        let expected = json!({
            "id": 2,
            "value": "new"
        });

        assert_eq!(result, expected);
    }
}
use serde_json::{Value, Map};
use std::collections::HashSet;

pub enum MergeStrategy {
    PreferFirst,
    PreferSecond,
    CombineArrays,
    FailOnConflict,
}

pub fn merge_json(a: &Value, b: &Value, strategy: &MergeStrategy) -> Result<Value, String> {
    match (a, b) {
        (Value::Object(map_a), Value::Object(map_b)) => merge_objects(map_a, map_b, strategy),
        (Value::Array(arr_a), Value::Array(arr_b)) => merge_arrays(arr_a, arr_b, strategy),
        _ => {
            if a == b {
                Ok(a.clone())
            } else {
                handle_value_conflict(a, b, strategy)
            }
        }
    }
}

fn merge_objects(
    a: &Map<String, Value>,
    b: &Map<String, Value>,
    strategy: &MergeStrategy,
) -> Result<Value, String> {
    let mut result = Map::new();
    let keys_a: HashSet<_> = a.keys().collect();
    let keys_b: HashSet<_> = b.keys().collect();
    
    for key in keys_a.union(&keys_b) {
        let key_str = (*key).clone();
        match (a.get(&key_str), b.get(&key_str)) {
            (Some(val_a), Some(val_b)) => {
                let merged = merge_json(val_a, val_b, strategy)?;
                result.insert(key_str, merged);
            }
            (Some(val), None) | (None, Some(val)) => {
                result.insert(key_str, val.clone());
            }
            (None, None) => unreachable!(),
        }
    }
    
    Ok(Value::Object(result))
}

fn merge_arrays(
    a: &[Value],
    b: &[Value],
    strategy: &MergeStrategy,
) -> Result<Value, String> {
    match strategy {
        MergeStrategy::CombineArrays => {
            let mut combined = Vec::with_capacity(a.len() + b.len());
            combined.extend_from_slice(a);
            combined.extend_from_slice(b);
            Ok(Value::Array(combined))
        }
        _ => {
            if a == b {
                Ok(Value::Array(a.to_vec()))
            } else {
                handle_value_conflict(&Value::Array(a.to_vec()), &Value::Array(b.to_vec()), strategy)
            }
        }
    }
}

fn handle_value_conflict(a: &Value, b: &Value, strategy: &MergeStrategy) -> Result<Value, String> {
    match strategy {
        MergeStrategy::PreferFirst => Ok(a.clone()),
        MergeStrategy::PreferSecond => Ok(b.clone()),
        MergeStrategy::FailOnConflict => Err(format!("Conflict between values: {} and {}", a, b)),
        MergeStrategy::CombineArrays => {
            if let (Value::Array(arr_a), Value::Array(arr_b)) = (a, b) {
                merge_arrays(arr_a, arr_b, strategy)
            } else {
                Err("Cannot combine non-array values".to_string())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_merge_objects_prefer_first() {
        let a = json!({"x": 1, "y": 2});
        let b = json!({"x": 3, "z": 4});
        let result = merge_json(&a, &b, &MergeStrategy::PreferFirst).unwrap();
        assert_eq!(result["x"], 1);
        assert_eq!(result["y"], 2);
        assert_eq!(result["z"], 4);
    }

    #[test]
    fn test_merge_arrays_combine() {
        let a = json!([1, 2]);
        let b = json!([3, 4]);
        let result = merge_json(&a, &b, &MergeStrategy::CombineArrays).unwrap();
        assert_eq!(result, json!([1, 2, 3, 4]));
    }
}