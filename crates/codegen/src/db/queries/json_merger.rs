
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
        Some(existing) => {
            if let (Value::Object(ref mut existing_obj), Value::Object(new_obj)) = (existing, &new_value) {
                for (nested_key, nested_value) in new_obj {
                    merge_value(existing_obj, nested_key.clone(), nested_value.clone());
                }
            } else if existing != &new_value {
                let conflict_key = format!("{}_conflict", key);
                let conflict_array = match map.get_mut(&conflict_key) {
                    Some(Value::Array(arr)) => arr,
                    _ => {
                        let arr = vec![existing.clone()];
                        map.insert(conflict_key.clone(), Value::Array(arr));
                        map.get_mut(&conflict_key).unwrap().as_array_mut().unwrap()
                    }
                };
                conflict_array.push(new_value);
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
    fn test_basic_merge() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();

        fs::write(&file1, r#"{"a": 1, "b": {"c": 2}}"#).unwrap();
        fs::write(&file2, r#"{"b": {"d": 3}, "e": 4}"#).unwrap();

        let result = merge_json_files(&[&file1, &file2]).unwrap();
        let expected = json!({
            "a": 1,
            "b": {"c": 2, "d": 3},
            "e": 4
        });

        assert_eq!(result, expected);
    }

    #[test]
    fn test_conflict_resolution() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();

        fs::write(&file1, r#"{"version": "1.0.0"}"#).unwrap();
        fs::write(&file2, r#"{"version": "2.0.0"}"#).unwrap();

        let result = merge_json_files(&[&file1, &file2]).unwrap();
        
        assert!(result.get("version").is_some());
        assert!(result.get("version_conflict").is_some());
        
        if let Some(Value::Array(arr)) = result.get("version_conflict") {
            assert_eq!(arr.len(), 1);
            assert!(arr.contains(&json!("2.0.0")));
        } else {
            panic!("Expected conflict array");
        }
    }
}