use serde_json::{json, Value};
use std::fs;
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P], output_path: P) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged_array = Vec::new();

    for path in paths {
        let content = fs::read_to_string(path)?;
        let parsed: Value = serde_json::from_str(&content)?;
        merged_array.push(parsed);
    }

    let output_value = Value::Array(merged_array);
    let output_json = serde_json::to_string_pretty(&output_value)?;
    fs::write(output_path, output_json)?;

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

        fs::write(&file1, r#"{"id": 1, "name": "test1"}"#).unwrap();
        fs::write(&file2, r#"{"id": 2, "name": "test2"}"#).unwrap();

        let paths = [file1.path(), file2.path()];
        merge_json_files(&paths, output_file.path()).unwrap();

        let output_content = fs::read_to_string(output_file.path()).unwrap();
        let expected = r#"[
  {
    "id": 1,
    "name": "test1"
  },
  {
    "id": 2,
    "name": "test2"
  }
]"#;
        assert_eq!(output_content.trim(), expected.trim());
    }
}
use serde_json::{Value, Map};
use std::fs;
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P]) -> Result<Value, Box<dyn std::error::Error>> {
    let mut merged_map = Map::new();

    for path in paths {
        let content = fs::read_to_string(path)?;
        let json_value: Value = serde_json::from_str(&content)?;

        if let Value::Object(map) = json_value {
            for (key, value) in map {
                merge_value(&mut merged_map, key, value);
            }
        }
    }

    Ok(Value::Object(merged_map))
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
            } else if existing_value.is_array() && new_value.is_array() {
                if let (Value::Array(existing_arr), Value::Array(new_arr)) = (existing_value, new_value) {
                    existing_arr.extend(new_arr);
                }
            } else {
                map.insert(key, new_value);
            }
        }
        None => {
            map.insert(key, new_value);
        }
    }
}
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, Read, Write};
use std::path::Path;

use serde_json::{json, Value};

pub fn merge_json_files(input_paths: &[&str], output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged_array = Vec::new();

    for input_path in input_paths {
        let path = Path::new(input_path);
        if !path.exists() {
            eprintln!("Warning: File {} does not exist, skipping.", input_path);
            continue;
        }

        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        let mut contents = String::new();
        reader.read_to_string(&mut contents)?;

        let json_value: Value = serde_json::from_str(&contents)?;

        match json_value {
            Value::Array(arr) => {
                merged_array.extend(arr);
            }
            Value::Object(_) => {
                merged_array.push(json_value);
            }
            _ => {
                eprintln!("Warning: File {} does not contain a JSON object or array, skipping.", input_path);
            }
        }
    }

    let output_value = Value::Array(merged_array);
    let output_json = serde_json::to_string_pretty(&output_value)?;

    let mut output_file = File::create(output_path)?;
    output_file.write_all(output_json.as_bytes())?;

    Ok(())
}

pub fn merge_json_with_deduplication(
    input_paths: &[&str],
    output_path: &str,
    key_field: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut unique_map: HashMap<String, Value> = HashMap::new();

    for input_path in input_paths {
        let path = Path::new(input_path);
        if !path.exists() {
            eprintln!("Warning: File {} does not exist, skipping.", input_path);
            continue;
        }

        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        let mut contents = String::new();
        reader.read_to_string(&mut contents)?;

        let json_value: Value = serde_json::from_str(&contents)?;

        match json_value {
            Value::Array(arr) => {
                for item in arr {
                    if let Some(obj) = item.as_object() {
                        if let Some(key_value) = obj.get(key_field) {
                            if let Some(key_str) = key_value.as_str() {
                                unique_map.insert(key_str.to_string(), item.clone());
                            }
                        }
                    }
                }
            }
            Value::Object(obj) => {
                if let Some(key_value) = obj.get(key_field) {
                    if let Some(key_str) = key_value.as_str() {
                        unique_map.insert(key_str.to_string(), Value::Object(obj.clone()));
                    }
                }
            }
            _ => {
                eprintln!("Warning: File {} does not contain a JSON object or array, skipping.", input_path);
            }
        }
    }

    let merged_array: Vec<Value> = unique_map.into_values().collect();
    let output_value = Value::Array(merged_array);
    let output_json = serde_json::to_string_pretty(&output_value)?;

    let mut output_file = File::create(output_path)?;
    output_file.write_all(output_json.as_bytes())?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_files() {
        let json1 = r#"[{"id": 1, "name": "Alice"}, {"id": 2, "name": "Bob"}]"#;
        let json2 = r#"{"id": 3, "name": "Charlie"}"#;

        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();
        file1.write_all(json1.as_bytes()).unwrap();
        file2.write_all(json2.as_bytes()).unwrap();

        let output_file = NamedTempFile::new().unwrap();
        let output_path = output_file.path().to_str().unwrap();

        let input_paths = vec![
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ];

        let result = merge_json_files(&input_paths, output_path);
        assert!(result.is_ok());

        let output_content = fs::read_to_string(output_path).unwrap();
        let parsed: Value = serde_json::from_str(&output_content).unwrap();
        assert!(parsed.is_array());
        let array = parsed.as_array().unwrap();
        assert_eq!(array.len(), 3);
    }

    #[test]
    fn test_merge_json_with_deduplication() {
        let json1 = r#"[{"id": "a1", "name": "Alice"}, {"id": "a2", "name": "Bob"}]"#;
        let json2 = r#"[{"id": "a1", "name": "Alice Updated"}, {"id": "a3", "name": "Charlie"}]"#;

        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();
        file1.write_all(json1.as_bytes()).unwrap();
        file2.write_all(json2.as_bytes()).unwrap();

        let output_file = NamedTempFile::new().unwrap();
        let output_path = output_file.path().to_str().unwrap();

        let input_paths = vec![
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ];

        let result = merge_json_with_deduplication(&input_paths, output_path, "id");
        assert!(result.is_ok());

        let output_content = fs::read_to_string(output_path).unwrap();
        let parsed: Value = serde_json::from_str(&output_content).unwrap();
        let array = parsed.as_array().unwrap();
        assert_eq!(array.len(), 3);

        let mut ids: Vec<String> = array
            .iter()
            .filter_map(|v| v.get("id").and_then(|id| id.as_str()).map(|s| s.to_string()))
            .collect();
        ids.sort();
        assert_eq!(ids, vec!["a1", "a2", "a3"]);
    }
}