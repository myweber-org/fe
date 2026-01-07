
use serde_json::{Value, Map};
use std::fs;
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P]) -> Result<Value, String> {
    let mut merged = Map::new();

    for path in paths {
        let content = fs::read_to_string(path).map_err(|e| format!("Failed to read {}: {}", path.as_ref().display(), e))?;
        let json: Value = serde_json::from_str(&content).map_err(|e| format!("Invalid JSON in {}: {}", path.as_ref().display(), e))?;

        if let Value::Object(obj) = json {
            for (key, value) in obj {
                if merged.contains_key(&key) {
                    let existing = merged.get(&key).unwrap();
                    if existing != &value {
                        return Err(format!("Conflict detected for key '{}'", key));
                    }
                } else {
                    merged.insert(key, value);
                }
            }
        } else {
            return Err("Top-level JSON must be an object".to_string());
        }
    }

    Ok(Value::Object(merged))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_files() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();

        fs::write(&file1, r#"{"a": 1, "b": 2}"#).unwrap();
        fs::write(&file2, r#"{"c": 3, "d": 4}"#).unwrap();

        let result = merge_json_files(&[file1.path(), file2.path()]).unwrap();
        let expected: Value = serde_json::from_str(r#"{"a": 1, "b": 2, "c": 3, "d": 4}"#).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_merge_json_conflict() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();

        fs::write(&file1, r#"{"a": 1}"#).unwrap();
        fs::write(&file2, r#"{"a": 2}"#).unwrap();

        let result = merge_json_files(&[file1.path(), file2.path()]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Conflict"));
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
            if let (Value::Object(existing_obj), Value::Object(new_obj)) = (existing, &new_value) {
                for (nested_key, nested_value) in new_obj {
                    merge_value(existing_obj, nested_key.clone(), nested_value.clone());
                }
            } else if existing != &new_value {
                *existing = Value::Array(vec![existing.clone(), new_value]);
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
    fn test_merge_json() {
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

    #[test]
    fn test_merge_conflict() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();

        fs::write(&file1, r#"{"a": 1}"#).unwrap();
        fs::write(&file2, r#"{"a": 2}"#).unwrap();

        let result = merge_json_files(&[file1.path(), file2.path()]).unwrap();
        let expected = json!({
            "a": [1, 2]
        });

        assert_eq!(result, expected);
    }
}