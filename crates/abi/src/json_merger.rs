
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
    match map.get(&key) {
        Some(existing) => {
            match (existing, new_value) {
                (Value::Object(old_obj), Value::Object(new_obj)) => {
                    let mut merged_obj = old_obj.clone();
                    for (k, v) in new_obj {
                        merge_value(&mut merged_obj, k, v);
                    }
                    map.insert(key, Value::Object(merged_obj));
                }
                (Value::Array(old_arr), Value::Array(new_arr)) => {
                    let mut merged_arr = old_arr.clone();
                    merged_arr.extend(new_arr);
                    map.insert(key, Value::Array(merged_arr));
                }
                (old, new) => {
                    if old != &new {
                        eprintln!("Conflict detected for key '{}', using new value", key);
                        map.insert(key, new);
                    }
                }
            }
        }
        None => {
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
    fn test_merge_basic() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();

        fs::write(&file1, r#"{"name": "Alice", "age": 30}"#).unwrap();
        fs::write(&file2, r#"{"city": "Berlin", "age": 31}"#).unwrap();

        let result = merge_json_files(&[file1.path(), file2.path()]).unwrap();
        let expected = json!({
            "name": "Alice",
            "age": 31,
            "city": "Berlin"
        });

        assert_eq!(result, expected);
    }

    #[test]
    fn test_merge_nested() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();

        fs::write(&file1, r#"{"user": {"name": "Bob"}, "tags": ["rust"]}"#).unwrap();
        fs::write(&file2, r#"{"user": {"age": 25}, "tags": ["json"]}"#).unwrap();

        let result = merge_json_files(&[file1.path(), file2.path()]).unwrap();
        let expected = json!({
            "user": {
                "name": "Bob",
                "age": 25
            },
            "tags": ["rust", "json"]
        });

        assert_eq!(result, expected);
    }
}use serde_json::{Map, Value};
use std::fs;
use std::path::Path;

pub fn merge_json_files(file_paths: &[&str]) -> Result<Value, Box<dyn std::error::Error>> {
    let mut merged_map = Map::new();

    for path_str in file_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            continue;
        }

        let content = fs::read_to_string(path)?;
        let json_value: Value = serde_json::from_str(&content)?;

        if let Value::Object(map) = json_value {
            for (key, value) in map {
                merged_map.insert(key, value);
            }
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
}