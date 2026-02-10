
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

pub fn find_common_keys(first: &Map<String, Value>, second: &Map<String, Value>) -> HashSet<String> {
    first.keys().filter(|k| second.contains_key(*k)).cloned().collect()
}use serde_json::{Map, Value};
use std::fs;
use std::path::Path;

pub fn merge_json_files(file_paths: &[&str]) -> Result<Value, Box<dyn std::error::Error>> {
    let mut merged_map = Map::new();

    for path_str in file_paths {
        let path = Path::new(path_str);
        let content = fs::read_to_string(path)?;
        let json_value: Value = serde_json::from_str(&content)?;

        if let Value::Object(map) = json_value {
            for (key, value) in map {
                merged_map.insert(key, value);
            }
        } else {
            return Err("Each JSON file must contain a JSON object".into());
        }
    }

    Ok(Value::Object(merged_map))
}