
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, Read, Write};
use std::path::Path;

use serde_json::{Value, json};

pub fn merge_json_files(input_paths: &[String], output_path: &str) -> Result<(), String> {
    let mut merged_array = Vec::new();

    for path_str in input_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            return Err(format!("File not found: {}", path_str));
        }

        let mut file = File::open(path).map_err(|e| e.to_string())?;
        let mut contents = String::new();
        file.read_to_string(&mut contents).map_err(|e| e.to_string())?;

        let json_value: Value = serde_json::from_str(&contents)
            .map_err(|e| format!("Failed to parse JSON from {}: {}", path_str, e))?;

        match json_value {
            Value::Array(arr) => {
                merged_array.extend(arr);
            }
            Value::Object(obj) => {
                merged_array.push(Value::Object(obj));
            }
            _ => {
                return Err(format!("JSON in {} is not an array or object", path_str));
            }
        }
    }

    let output_file = File::create(output_path).map_err(|e| e.to_string())?;
    serde_json::to_writer_pretty(output_file, &json!(merged_array))
        .map_err(|e| e.to_string())?;

    Ok(())
}

pub fn merge_with_key_deduplication(
    input_paths: &[String],
    output_path: &str,
    key_field: &str,
) -> Result<(), String> {
    let mut unique_map: HashMap<String, Value> = HashMap::new();

    for path_str in input_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            return Err(format!("File not found: {}", path_str));
        }

        let file = File::open(path).map_err(|e| e.to_string())?;
        let reader = BufReader::new(file);
        let json_value: Value = serde_json::from_reader(reader)
            .map_err(|e| format!("Failed to parse JSON from {}: {}", path_str, e))?;

        match json_value {
            Value::Array(arr) => {
                for item in arr {
                    if let Value::Object(map) = item {
                        if let Some(key_value) = map.get(key_field) {
                            if let Some(key_str) = key_value.as_str() {
                                unique_map.insert(key_str.to_string(), Value::Object(map));
                            }
                        }
                    }
                }
            }
            Value::Object(obj) => {
                if let Some(key_value) = obj.get(key_field) {
                    if let Some(key_str) = key_value.as_str() {
                        unique_map.insert(key_str.to_string(), Value::Object(obj));
                    }
                }
            }
            _ => {
                return Err(format!("JSON in {} is not an array or object", path_str));
            }
        }
    }

    let unique_values: Vec<Value> = unique_map.into_values().collect();
    let output_file = File::create(output_path).map_err(|e| e.to_string())?;
    serde_json::to_writer_pretty(output_file, &json!(unique_values))
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_arrays() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();

        fs::write(file1.path(), r#"[{"id": 1}, {"id": 2}]"#).unwrap();
        fs::write(file2.path(), r#"[{"id": 3}, {"id": 4}]"#).unwrap();

        let inputs = vec![
            file1.path().to_str().unwrap().to_string(),
            file2.path().to_str().unwrap().to_string(),
        ];

        let result = merge_json_files(&inputs, output_file.path().to_str().unwrap());
        assert!(result.is_ok());

        let content = fs::read_to_string(output_file.path()).unwrap();
        let parsed: Value = serde_json::from_str(&content).unwrap();
        assert_eq!(parsed.as_array().unwrap().len(), 4);
    }

    #[test]
    fn test_deduplication() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();

        fs::write(file1.path(), r#"[{"id": "a", "value": 1}, {"id": "b", "value": 2}]"#).unwrap();
        fs::write(file2.path(), r#"[{"id": "a", "value": 3}, {"id": "c", "value": 4}]"#).unwrap();

        let inputs = vec![
            file1.path().to_str().unwrap().to_string(),
            file2.path().to_str().unwrap().to_string(),
        ];

        let result = merge_with_key_deduplication(
            &inputs,
            output_file.path().to_str().unwrap(),
            "id",
        );
        assert!(result.is_ok());

        let content = fs::read_to_string(output_file.path()).unwrap();
        let parsed: Value = serde_json::from_str(&content).unwrap();
        assert_eq!(parsed.as_array().unwrap().len(), 3);
    }
}