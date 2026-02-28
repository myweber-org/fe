use std::collections::HashMap;
use serde_json::{Value, Map};

pub fn merge_json(base: &mut Value, extension: &Value) {
    match (base, extension) {
        (Value::Object(base_map), Value::Object(ext_map)) => {
            for (key, ext_value) in ext_map {
                if let Some(base_value) = base_map.get_mut(key) {
                    merge_json(base_value, ext_value);
                } else {
                    base_map.insert(key.clone(), ext_value.clone());
                }
            }
        }
        (base, extension) => {
            *base = extension.clone();
        }
    }
}

pub fn merge_multiple_json(objects: Vec<Value>) -> Option<Value> {
    let mut result = Value::Object(Map::new());
    for obj in objects {
        merge_json(&mut result, &obj);
    }
    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_basic_merge() {
        let mut base = json!({"a": 1, "b": {"c": 2}});
        let extension = json!({"b": {"d": 3}, "e": 4});
        merge_json(&mut base, &extension);
        assert_eq!(base, json!({"a": 1, "b": {"c": 2, "d": 3}, "e": 4}));
    }

    #[test]
    fn test_overwrite_primitive() {
        let mut base = json!({"a": 1});
        let extension = json!({"a": 2});
        merge_json(&mut base, &extension);
        assert_eq!(base, json!({"a": 2}));
    }

    #[test]
    fn test_multiple_merge() {
        let objects = vec![
            json!({"a": 1}),
            json!({"b": 2}),
            json!({"a": 3, "c": 4})
        ];
        let result = merge_multiple_json(objects).unwrap();
        assert_eq!(result, json!({"a": 3, "b": 2, "c": 4}));
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
                if merged_map.contains_key(&key) {
                    eprintln!("Warning: Key '{}' already exists, overwriting with value from {}", key, path_str);
                }
                merged_map.insert(key, value);
            }
        } else {
            return Err("Top-level JSON must be an object".into());
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
        writeln!(file2, r#"{"city": "Berlin", "age": 31}"#).unwrap();

        let paths = [
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ];

        let result = merge_json_files(&paths).unwrap();
        let obj = result.as_object().unwrap();

        assert_eq!(obj.get("name").unwrap().as_str().unwrap(), "Alice");
        assert_eq!(obj.get("city").unwrap().as_str().unwrap(), "Berlin");
        assert_eq!(obj.get("age").unwrap().as_i64().unwrap(), 31);
    }

    #[test]
    fn test_file_not_found() {
        let result = merge_json_files(&["nonexistent.json"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_json() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "invalid json").unwrap();

        let paths = [file.path().to_str().unwrap()];
        let result = merge_json_files(&paths);
        assert!(result.is_err());
    }
}