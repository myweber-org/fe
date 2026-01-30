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
            return Err("Each JSON file must contain a JSON object at its root.".into());
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

        writeln!(file1, r#"{{ "a": 1, "b": "test" }}"#).unwrap();
        writeln!(file2, r#"{{ "c": true, "d": [1,2,3] }}"#).unwrap();

        let paths = [
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ];

        let result = merge_json_files(&paths).unwrap();
        let expected = json!({
            "a": 1,
            "b": "test",
            "c": true,
            "d": [1, 2, 3]
        });

        assert_eq!(result, expected);
    }

    #[test]
    fn test_overwrite_keys() {
        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();

        writeln!(file1, r#"{{ "key": "first" }}"#).unwrap();
        writeln!(file2, r#"{{ "key": "second" }}"#).unwrap();

        let paths = [
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ];

        let result = merge_json_files(&paths).unwrap();
        assert_eq!(result["key"], "second");
    }
}
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, Read};
use std::path::Path;

use serde_json::{json, Value};

pub fn merge_json_files(file_paths: &[&str], output_path: &str) -> Result<(), String> {
    let mut merged_array = Vec::new();

    for file_path in file_paths {
        let path = Path::new(file_path);
        if !path.exists() {
            return Err(format!("File not found: {}", file_path));
        }

        let mut file = File::open(path).map_err(|e| e.to_string())?;
        let mut contents = String::new();
        file.read_to_string(&mut contents).map_err(|e| e.to_string())?;

        let json_value: Value = serde_json::from_str(&contents).map_err(|e| e.to_string())?;

        match json_value {
            Value::Array(arr) => {
                merged_array.extend(arr);
            }
            Value::Object(obj) => {
                merged_array.push(Value::Object(obj));
            }
            _ => {
                return Err(format!("Unsupported JSON structure in file: {}", file_path));
            }
        }
    }

    let output_value = Value::Array(merged_array);
    let output_json = serde_json::to_string_pretty(&output_value).map_err(|e| e.to_string())?;

    fs::write(output_path, output_json).map_err(|e| e.to_string())?;

    Ok(())
}

pub fn merge_json_with_deduplication(
    file_paths: &[&str],
    output_path: &str,
    unique_key: &str,
) -> Result<(), String> {
    let mut unique_map: HashMap<String, Value> = HashMap::new();

    for file_path in file_paths {
        let path = Path::new(file_path);
        if !path.exists() {
            return Err(format!("File not found: {}", file_path));
        }

        let file = File::open(path).map_err(|e| e.to_string())?;
        let reader = BufReader::new(file);
        let json_value: Value = serde_json::from_reader(reader).map_err(|e| e.to_string())?;

        match json_value {
            Value::Array(arr) => {
                for item in arr {
                    if let Value::Object(map) = item {
                        if let Some(Value::String(key_value)) = map.get(unique_key) {
                            unique_map.insert(key_value.clone(), Value::Object(map));
                        }
                    }
                }
            }
            Value::Object(obj) => {
                if let Some(Value::String(key_value)) = obj.get(unique_key) {
                    unique_map.insert(key_value.clone(), Value::Object(obj));
                }
            }
            _ => {
                return Err(format!("Unsupported JSON structure in file: {}", file_path));
            }
        }
    }

    let deduplicated_array: Vec<Value> = unique_map.into_values().collect();
    let output_value = Value::Array(deduplicated_array);
    let output_json = serde_json::to_string_pretty(&output_value).map_err(|e| e.to_string())?;

    fs::write(output_path, output_json).map_err(|e| e.to_string())?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_files() {
        let file1_content = r#"[{"id": 1, "name": "Alice"}, {"id": 2, "name": "Bob"}]"#;
        let file2_content = r#"[{"id": 3, "name": "Charlie"}]"#;

        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();

        fs::write(file1.path(), file1_content).unwrap();
        fs::write(file2.path(), file2_content).unwrap();

        let result = merge_json_files(
            &[
                file1.path().to_str().unwrap(),
                file2.path().to_str().unwrap(),
            ],
            output_file.path().to_str().unwrap(),
        );

        assert!(result.is_ok());

        let output_content = fs::read_to_string(output_file.path()).unwrap();
        let parsed: Value = serde_json::from_str(&output_content).unwrap();

        assert!(parsed.is_array());
        let array = parsed.as_array().unwrap();
        assert_eq!(array.len(), 3);
    }

    #[test]
    fn test_merge_json_with_deduplication() {
        let file1_content = r#"[{"id": "1", "name": "Alice"}, {"id": "2", "name": "Bob"}]"#;
        let file2_content = r#"[{"id": "1", "name": "Alice Updated"}, {"id": "3", "name": "Charlie"}]"#;

        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();

        fs::write(file1.path(), file1_content).unwrap();
        fs::write(file2.path(), file2_content).unwrap();

        let result = merge_json_with_deduplication(
            &[
                file1.path().to_str().unwrap(),
                file2.path().to_str().unwrap(),
            ],
            output_file.path().to_str().unwrap(),
            "id",
        );

        assert!(result.is_ok());

        let output_content = fs::read_to_string(output_file.path()).unwrap();
        let parsed: Value = serde_json::from_str(&output_content).unwrap();

        assert!(parsed.is_array());
        let array = parsed.as_array().unwrap();
        assert_eq!(array.len(), 3);

        let mut ids: Vec<String> = array
            .iter()
            .filter_map(|item| item.get("id").and_then(|id| id.as_str()).map(String::from))
            .collect();
        ids.sort();
        assert_eq!(ids, vec!["1", "2", "3"]);
    }
}