
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub fn merge_json_files(file_paths: &[&str]) -> Result<Value, String> {
    let mut merged_map = Map::new();

    for path_str in file_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            return Err(format!("File not found: {}", path_str));
        }

        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read {}: {}", path_str, e))?;

        let json_value: Value = serde_json::from_str(&content)
            .map_err(|e| format!("Invalid JSON in {}: {}", path_str, e))?;

        if let Value::Object(obj) = json_value {
            for (key, value) in obj {
                if merged_map.contains_key(&key) {
                    return Err(format!("Duplicate key '{}' found in {}", key, path_str));
                }
                merged_map.insert(key, value);
            }
        } else {
            return Err(format!("Top-level JSON must be an object in {}", path_str));
        }
    }

    Ok(Value::Object(merged_map))
}

pub fn merge_json_with_strategy(
    file_paths: &[&str],
    conflict_strategy: ConflictStrategy,
) -> Result<Value, String> {
    let mut accumulator: HashMap<String, Value> = HashMap::new();

    for path_str in file_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            return Err(format!("File not found: {}", path_str));
        }

        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read {}: {}", path_str, e))?;

        let json_value: Value = serde_json::from_str(&content)
            .map_err(|e| format!("Invalid JSON in {}: {}", path_str, e))?;

        if let Value::Object(obj) = json_value {
            for (key, value) in obj {
                match conflict_strategy {
                    ConflictStrategy::Overwrite => {
                        accumulator.insert(key.clone(), value);
                    }
                    ConflictStrategy::Skip => {
                        accumulator.entry(key.clone()).or_insert(value);
                    }
                    ConflictStrategy::Error => {
                        if accumulator.contains_key(&key) {
                            return Err(format!(
                                "Duplicate key '{}' found in {}",
                                key, path_str
                            ));
                        }
                        accumulator.insert(key.clone(), value);
                    }
                }
            }
        } else {
            return Err(format!("Top-level JSON must be an object in {}", path_str));
        }
    }

    let mut map = Map::new();
    for (key, value) in accumulator {
        map.insert(key, value);
    }
    Ok(Value::Object(map))
}

pub enum ConflictStrategy {
    Overwrite,
    Skip,
    Error,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_temp_json(content: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "{}", content).unwrap();
        file
    }

    #[test]
    fn test_merge_json_files() {
        let file1 = create_temp_json(r#"{"a": 1, "b": 2}"#);
        let file2 = create_temp_json(r#"{"c": 3, "d": 4}"#);

        let result = merge_json_files(&[
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ]);

        assert!(result.is_ok());
        let merged = result.unwrap();
        assert_eq!(merged["a"], 1);
        assert_eq!(merged["b"], 2);
        assert_eq!(merged["c"], 3);
        assert_eq!(merged["d"], 4);
    }

    #[test]
    fn test_merge_with_duplicate_error() {
        let file1 = create_temp_json(r#"{"a": 1}"#);
        let file2 = create_temp_json(r#"{"a": 2}"#);

        let result = merge_json_files(&[
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ]);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Duplicate key"));
    }

    #[test]
    fn test_merge_with_overwrite_strategy() {
        let file1 = create_temp_json(r#"{"a": 1}"#);
        let file2 = create_temp_json(r#"{"a": 2}"#);

        let result = merge_json_with_strategy(
            &[
                file1.path().to_str().unwrap(),
                file2.path().to_str().unwrap(),
            ],
            ConflictStrategy::Overwrite,
        );

        assert!(result.is_ok());
        let merged = result.unwrap();
        assert_eq!(merged["a"], 2);
    }
}
use serde_json::{Value, Map};
use std::collections::HashSet;

pub enum ConflictResolution {
    PreferFirst,
    PreferSecond,
    MergeArrays,
    FailOnConflict,
}

pub fn merge_json_objects(
    first: &Value,
    second: &Value,
    resolution: ConflictResolution,
) -> Result<Value, String> {
    if !first.is_object() || !second.is_object() {
        return Err("Both inputs must be JSON objects".to_string());
    }

    let mut result = Map::new();
    let first_obj = first.as_object().unwrap();
    let second_obj = second.as_object().unwrap();

    let first_keys: HashSet<_> = first_obj.keys().collect();
    let second_keys: HashSet<_> = second_obj.keys().collect();

    for key in first_keys.union(&second_keys) {
        let key_str = key.to_string();
        
        match (first_obj.get(key), second_obj.get(key)) {
            (Some(v1), Some(v2)) => {
                if v1 == v2 {
                    result.insert(key_str, v1.clone());
                } else {
                    match resolution {
                        ConflictResolution::PreferFirst => {
                            result.insert(key_str, v1.clone());
                        }
                        ConflictResolution::PreferSecond => {
                            result.insert(key_str, v2.clone());
                        }
                        ConflictResolution::MergeArrays => {
                            if v1.is_array() && v2.is_array() {
                                let mut merged_array = v1.as_array().unwrap().clone();
                                merged_array.extend(v2.as_array().unwrap().clone());
                                result.insert(key_str, Value::Array(merged_array));
                            } else {
                                return Err(format!(
                                    "Conflict on key '{}': both values are not arrays",
                                    key
                                ));
                            }
                        }
                        ConflictResolution::FailOnConflict => {
                            return Err(format!("Conflict on key '{}'", key));
                        }
                    }
                }
            }
            (Some(v), None) => {
                result.insert(key_str, v.clone());
            }
            (None, Some(v)) => {
                result.insert(key_str, v.clone());
            }
            (None, None) => unreachable!(),
        }
    }

    Ok(Value::Object(result))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_merge_prefer_first() {
        let first = json!({"a": 1, "b": 2});
        let second = json!({"b": 3, "c": 4});
        
        let result = merge_json_objects(&first, &second, ConflictResolution::PreferFirst)
            .unwrap();
        
        assert_eq!(result, json!({"a": 1, "b": 2, "c": 4}));
    }

    #[test]
    fn test_merge_arrays() {
        let first = json!({"items": [1, 2]});
        let second = json!({"items": [3, 4]});
        
        let result = merge_json_objects(&first, &second, ConflictResolution::MergeArrays)
            .unwrap();
        
        assert_eq!(result, json!({"items": [1, 2, 3, 4]}));
    }
}