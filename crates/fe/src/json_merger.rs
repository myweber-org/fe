
use serde_json::{Value, Map};
use std::fs;
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P]) -> Result<Value, String> {
    if paths.is_empty() {
        return Err("No input files provided".to_string());
    }

    let mut merged = Map::new();

    for path in paths {
        let content = fs::read_to_string(path).map_err(|e| format!("Failed to read {}: {}", path.as_ref().display(), e))?;
        let json: Value = serde_json::from_str(&content).map_err(|e| format!("Invalid JSON in {}: {}", path.as_ref().display(), e))?;

        if let Value::Object(obj) = json {
            merge_objects(&mut merged, obj);
        } else {
            return Err(format!("Top-level element in {} must be a JSON object", path.as_ref().display()));
        }
    }

    Ok(Value::Object(merged))
}

fn merge_objects(target: &mut Map<String, Value>, source: Map<String, Value>) {
    for (key, source_value) in source {
        match target.get_mut(&key) {
            Some(target_value) => {
                if let (Value::Object(target_obj), Value::Object(source_obj)) = (target_value, &source_value) {
                    let mut target_map = target_obj.clone();
                    merge_objects(&mut target_map, source_obj.clone());
                    *target_value = Value::Object(target_map);
                } else if target_value != &source_value {
                    *target_value = Value::Array(vec![target_value.clone(), source_value]);
                }
            }
            None => {
                target.insert(key, source_value);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_basic() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();

        fs::write(&file1, r#"{"a": 1, "b": "test"}"#).unwrap();
        fs::write(&file2, r#"{"c": true, "d": null}"#).unwrap();

        let result = merge_json_files(&[file1.path(), file2.path()]).unwrap();
        let expected = json!({
            "a": 1,
            "b": "test",
            "c": true,
            "d": null
        });

        assert_eq!(result, expected);
    }

    #[test]
    fn test_merge_conflict() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();

        fs::write(&file1, r#"{"x": "first"}"#).unwrap();
        fs::write(&file2, r#"{"x": "second"}"#).unwrap();

        let result = merge_json_files(&[file1.path(), file2.path()]).unwrap();
        let expected = json!({
            "x": ["first", "second"]
        });

        assert_eq!(result, expected);
    }
}use std::fs::{self, File};
use std::io::{BufReader, Read};
use std::path::Path;

use serde_json::{json, Value};

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P]) -> Result<Value, Box<dyn std::error::Error>> {
    let mut merged_array = Vec::new();

    for path in paths {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        let mut contents = String::new();
        reader.read_to_string(&mut contents)?;

        let parsed: Value = serde_json::from_str(&contents)?;
        if let Value::Array(arr) = parsed {
            merged_array.extend(arr);
        } else {
            merged_array.push(parsed);
        }
    }

    Ok(json!(merged_array))
}

pub fn write_merged_json<P: AsRef<Path>>(output_path: P, value: &Value) -> std::io::Result<()> {
    let json_string = serde_json::to_string_pretty(value)?;
    fs::write(output_path, json_string)
}use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub fn merge_json_files(file_paths: &[&str], output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged_array = Vec::new();
    let mut seen_ids = HashMap::new();

    for file_path in file_paths {
        let path = Path::new(file_path);
        if !path.exists() {
            eprintln!("Warning: File {} not found, skipping.", file_path);
            continue;
        }

        let content = fs::read_to_string(path)?;
        let json_value: Value = serde_json::from_str(&content)?;

        match json_value {
            Value::Array(arr) => {
                for item in arr {
                    if let Some(obj) = item.as_object() {
                        if let Some(id_value) = obj.get("id") {
                            if let Some(id_str) = id_value.as_str() {
                                if seen_ids.contains_key(id_str) {
                                    eprintln!("Duplicate ID '{}' found, skipping.", id_str);
                                    continue;
                                }
                                seen_ids.insert(id_str.to_string(), ());
                            }
                        }
                    }
                    merged_array.push(item);
                }
            }
            Value::Object(_) => {
                if let Some(id_value) = json_value.get("id") {
                    if let Some(id_str) = id_value.as_str() {
                        if seen_ids.contains_key(id_str) {
                            eprintln!("Duplicate ID '{}' found, skipping.", id_str);
                            continue;
                        }
                        seen_ids.insert(id_str.to_string(), ());
                    }
                }
                merged_array.push(json_value);
            }
            _ => {
                eprintln!("Warning: JSON root in {} is not an array or object, skipping.", file_path);
            }
        }
    }

    let output_json = json!(merged_array);
    fs::write(output_path, output_json.to_string())?;
    println!("Successfully merged {} items into {}", merged_array.len(), output_path);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_files() {
        let file1_content = r#"[{"id": "1", "name": "Alice"}, {"id": "2", "name": "Bob"}]"#;
        let file2_content = r#"{"id": "3", "name": "Charlie"}"#;

        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();

        fs::write(file1.path(), file1_content).unwrap();
        fs::write(file2.path(), file2_content).unwrap();

        let paths = vec![file1.path().to_str().unwrap(), file2.path().to_str().unwrap()];
        let result = merge_json_files(&paths, output_file.path().to_str().unwrap());

        assert!(result.is_ok());

        let output_content = fs::read_to_string(output_file.path()).unwrap();
        let parsed: Value = serde_json::from_str(&output_content).unwrap();
        assert!(parsed.is_array());
        let arr = parsed.as_array().unwrap();
        assert_eq!(arr.len(), 3);
    }
}