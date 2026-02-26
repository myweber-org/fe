
use serde_json::{Value, Map};
use std::fs;
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P]) -> Result<Value, Box<dyn std::error::Error>> {
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

    Ok(Value::Object(merged))
}

fn merge_value(map: &mut Map<String, Value>, key: String, new_value: Value) {
    match map.get_mut(&key) {
        Some(existing) if existing.is_object() && new_value.is_object() => {
            if let (Value::Object(existing_obj), Value::Object(new_obj)) = (existing, new_value) {
                for (nested_key, nested_value) in new_obj {
                    merge_value(existing_obj, nested_key, nested_value);
                }
            }
        }
        Some(existing) if existing.is_array() && new_value.is_array() => {
            if let (Value::Array(existing_arr), Value::Array(new_arr)) = (existing, new_value) {
                existing_arr.extend(new_arr);
            }
        }
        _ => {
            map.insert(key, new_value);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_objects() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();

        fs::write(&file1, r#"{"a": 1, "b": {"x": 10}}"#).unwrap();
        fs::write(&file2, r#"{"b": {"y": 20}, "c": 3}"#).unwrap();

        let result = merge_json_files(&[file1.path(), file2.path()]).unwrap();
        let expected = json!({
            "a": 1,
            "b": {"x": 10, "y": 20},
            "c": 3
        });

        assert_eq!(result, expected);
    }
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
        merged_array.push(json_data);
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
        let json_data: JsonValue = serde_json::from_str(&contents)?;

        if let Some(obj) = json_data.as_object() {
            if let Some(value) = obj.get(key) {
                let key_str = value.as_str().unwrap_or_default().to_string();
                merged_map.insert(key_str, json_data.clone());
            }
        }
    }

    let result_map: serde_json::Map<String, JsonValue> = merged_map
        .into_iter()
        .map(|(k, v)| (k, v))
        .collect();

    Ok(JsonValue::Object(result_map))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_files() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();

        fs::write(&file1, r#"{"id": 1, "name": "Alice"}"#).unwrap();
        fs::write(&file2, r#"{"id": 2, "name": "Bob"}"#).unwrap();

        let paths = [file1.path(), file2.path()];
        let result = merge_json_files(&paths).unwrap();

        assert!(result.is_array());
        let array = result.as_array().unwrap();
        assert_eq!(array.len(), 2);
        assert_eq!(array[0]["name"], "Alice");
        assert_eq!(array[1]["name"], "Bob");
    }

    #[test]
    fn test_merge_json_with_key() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();

        fs::write(&file1, r#"{"id": "user_001", "score": 95}"#).unwrap();
        fs::write(&file2, r#"{"id": "user_002", "score": 87}"#).unwrap();

        let paths = [file1.path(), file2.path()];
        let result = merge_json_with_key(&paths, "id").unwrap();

        assert!(result.is_object());
        let obj = result.as_object().unwrap();
        assert_eq!(obj.len(), 2);
        assert_eq!(obj["user_001"]["score"], 95);
        assert_eq!(obj["user_002"]["score"], 87);
    }
}