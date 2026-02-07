use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub fn merge_json_files(file_paths: &[&str]) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let mut merged_map = HashMap::new();

    for path_str in file_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            continue;
        }

        let content = fs::read_to_string(path)?;
        let json_value: serde_json::Value = serde_json::from_str(&content)?;

        if let Some(obj) = json_value.as_object() {
            for (key, value) in obj {
                merged_map.insert(key.clone(), value.clone());
            }
        }
    }

    Ok(serde_json::Value::Object(merged_map.into_iter().collect()))
}

pub fn write_merged_json(output_path: &str, value: &serde_json::Value) -> Result<(), Box<dyn std::error::Error>> {
    let json_string = serde_json::to_string_pretty(value)?;
    fs::write(output_path, json_string)?;
    Ok(())
}use std::collections::HashMap;
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

        let json_data: JsonValue = serde_json::from_str(&contents)?;
        
        if let JsonValue::Array(arr) = json_data {
            merged_array.extend(arr);
        } else {
            merged_array.push(json_data);
        }
    }

    Ok(JsonValue::Array(merged_array))
}

pub fn merge_json_with_key(file_paths: &[impl AsRef<Path>], key: &str) -> Result<JsonValue, Box<dyn std::error::Error>> {
    let mut merged_map = HashMap::new();

    for path in file_paths {
        let contents = fs::read_to_string(path.as_ref())?;
        let json_data: JsonValue = serde_json::from_str(&contents)?;

        if let JsonValue::Object(obj) = json_data {
            if let Some(value) = obj.get(key) {
                merged_map.insert(key.to_string(), value.clone());
            }
        }
    }

    let result_object: serde_json::Map<String, JsonValue> = merged_map.into_iter().collect();
    Ok(JsonValue::Object(result_object))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_array_json() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();

        fs::write(&file1, r#"[{"id": 1}, {"id": 2}]"#).unwrap();
        fs::write(&file2, r#"[{"id": 3}, {"id": 4}]"#).unwrap();

        let result = merge_json_files(&[&file1, &file2]).unwrap();
        assert!(result.is_array());
        assert_eq!(result.as_array().unwrap().len(), 4);
    }

    #[test]
    fn test_merge_with_key() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();

        fs::write(&file1, r#"{"user": "alice", "age": 30}"#).unwrap();
        fs::write(&file2, r#"{"user": "bob", "age": 25}"#).unwrap();

        let result = merge_json_with_key(&[&file1, &file2], "user").unwrap();
        assert!(result.is_object());
        let obj = result.as_object().unwrap();
        assert!(obj.contains_key("user"));
    }
}