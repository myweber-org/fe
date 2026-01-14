
use std::collections::HashMap;
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
    use serde_json::json;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_files() {
        let file1_content = json!({"a": 1, "b": "test"});
        let file2_content = json!({"c": true, "d": [1, 2, 3]});

        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();

        fs::write(file1.path(), serde_json::to_string(&file1_content).unwrap()).unwrap();
        fs::write(file2.path(), serde_json::to_string(&file2_content).unwrap()).unwrap();

        let result = merge_json_files(&[file1.path().to_str().unwrap(), file2.path().to_str().unwrap()]);
        assert!(result.is_ok());

        let merged = result.unwrap();
        assert_eq!(merged["a"], 1);
        assert_eq!(merged["b"], "test");
        assert_eq!(merged["c"], true);
        assert_eq!(merged["d"], json!([1, 2, 3]));
    }

    #[test]
    fn test_merge_with_overwrite() {
        let file1_content = json!({"key": "original"});
        let file2_content = json!({"key": "overwritten"});

        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();

        fs::write(file1.path(), serde_json::to_string(&file1_content).unwrap()).unwrap();
        fs::write(file2.path(), serde_json::to_string(&file2_content).unwrap()).unwrap();

        let result = merge_json_files(&[file1.path().to_str().unwrap(), file2.path().to_str().unwrap()]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap()["key"], "overwritten");
    }
}
use serde_json::{Map, Value};
use std::collections::HashSet;

pub enum ConflictResolution {
    PreferFirst,
    PreferSecond,
    MergeArrays,
    FailOnConflict,
}

pub fn merge_json(
    first: &Map<String, Value>,
    second: &Map<String, Value>,
    resolution: ConflictResolution,
) -> Result<Map<String, Value>, String> {
    let mut result = first.clone();
    let mut conflicts = Vec::new();

    for (key, value2) in second {
        match result.get(key) {
            Some(value1) => {
                if value1 != value2 {
                    match resolution {
                        ConflictResolution::PreferFirst => continue,
                        ConflictResolution::PreferSecond => {
                            result.insert(key.clone(), value2.clone());
                        }
                        ConflictResolution::MergeArrays => {
                            if let (Value::Array(arr1), Value::Array(arr2)) = (value1, value2) {
                                let mut merged = arr1.clone();
                                merged.extend(arr2.clone());
                                result.insert(key.clone(), Value::Array(merged));
                            } else {
                                conflicts.push(key.clone());
                            }
                        }
                        ConflictResolution::FailOnConflict => {
                            return Err(format!("Conflict detected for key: {}", key));
                        }
                    }
                }
            }
            None => {
                result.insert(key.clone(), value2.clone());
            }
        }
    }

    if !conflicts.is_empty() && matches!(resolution, ConflictResolution::MergeArrays) {
        return Err(format!(
            "Cannot merge non-array values for keys: {:?}",
            conflicts
        ));
    }

    Ok(result)
}

pub fn find_common_keys(
    first: &Map<String, Value>,
    second: &Map<String, Value>,
) -> HashSet<String> {
    first.keys().filter(|k| second.contains_key(*k)).cloned().collect()
}use serde_json::{json, Value};
use std::fs;
use std::path::Path;

pub fn merge_json_files(file_paths: &[&str], output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged_array = Vec::new();

    for file_path in file_paths {
        let content = fs::read_to_string(file_path)?;
        let json_value: Value = serde_json::from_str(&content)?;
        merged_array.push(json_value);
    }

    let output_json = json!(merged_array);
    let output_dir = Path::new(output_path).parent().unwrap();
    if !output_dir.exists() {
        fs::create_dir_all(output_dir)?;
    }
    fs::write(output_path, output_json.to_string())?;
    Ok(())
}