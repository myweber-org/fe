
use serde_json::{Map, Value};
use std::collections::HashSet;

pub enum MergeStrategy {
    PreferFirst,
    PreferSecond,
    MergeObjects,
    MergeArrays,
    FailOnConflict,
}

pub fn merge_json(
    primary: &mut Value,
    secondary: &Value,
    strategy: &MergeStrategy,
) -> Result<(), String> {
    match (primary, secondary) {
        (Value::Object(primary_map), Value::Object(secondary_map)) => {
            merge_objects(primary_map, secondary_map, strategy)
        }
        (Value::Array(primary_arr), Value::Array(secondary_arr)) => {
            merge_arrays(primary_arr, secondary_arr, strategy)
        }
        _ => {
            if primary != secondary {
                match strategy {
                    MergeStrategy::PreferFirst => Ok(()),
                    MergeStrategy::PreferSecond => {
                        *primary = secondary.clone();
                        Ok(())
                    }
                    MergeStrategy::FailOnConflict => Err(format!(
                        "Conflict between values: {:?} and {:?}",
                        primary, secondary
                    )),
                    _ => Err("Cannot merge non-object/non-array values with this strategy".to_string()),
                }
            } else {
                Ok(())
            }
        }
    }
}

fn merge_objects(
    primary: &mut Map<String, Value>,
    secondary: &Map<String, Value>,
    strategy: &MergeStrategy,
) -> Result<(), String> {
    let primary_keys: HashSet<_> = primary.keys().collect();
    let secondary_keys: HashSet<_> = secondary.keys().collect();
    
    for key in secondary_keys.difference(&primary_keys) {
        if let Some(value) = secondary.get(*key) {
            primary.insert((*key).clone(), value.clone());
        }
    }
    
    for key in primary_keys.intersection(&secondary_keys) {
        let primary_value = primary.get_mut(*key).unwrap();
        let secondary_value = secondary.get(*key).unwrap();
        
        merge_json(primary_value, secondary_value, strategy)?;
    }
    
    Ok(())
}

fn merge_arrays(
    primary: &mut Vec<Value>,
    secondary: &Vec<Value>,
    strategy: &MergeStrategy,
) -> Result<(), String> {
    match strategy {
        MergeStrategy::MergeArrays => {
            primary.extend(secondary.iter().cloned());
            Ok(())
        }
        MergeStrategy::PreferFirst => Ok(()),
        MergeStrategy::PreferSecond => {
            *primary = secondary.clone();
            Ok(())
        }
        MergeStrategy::FailOnConflict => {
            if primary != secondary {
                Err("Array conflict detected".to_string())
            } else {
                Ok(())
            }
        }
        _ => Err("Cannot merge arrays with object merge strategy".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_merge_objects_prefer_first() {
        let mut primary = json!({"a": 1, "b": {"x": 10}});
        let secondary = json!({"a": 2, "b": {"y": 20}, "c": 3});
        
        merge_json(&mut primary, &secondary, &MergeStrategy::PreferFirst).unwrap();
        
        assert_eq!(primary["a"], 1);
        assert_eq!(primary["b"]["x"], 10);
        assert_eq!(primary["c"], 3);
    }

    #[test]
    fn test_merge_objects_merge_strategy() {
        let mut primary = json!({"a": {"x": 1}});
        let secondary = json!({"a": {"y": 2}});
        
        merge_json(&mut primary, &secondary, &MergeStrategy::MergeObjects).unwrap();
        
        assert_eq!(primary["a"]["x"], 1);
        assert_eq!(primary["a"]["y"], 2);
    }
}use serde_json::{Map, Value};
use std::fs;
use std::path::Path;

pub fn merge_json_files(file_paths: &[&str], output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged_map = Map::new();

    for file_path in file_paths {
        let content = fs::read_to_string(file_path)?;
        let json_value: Value = serde_json::from_str(&content)?;

        if let Value::Object(map) = json_value {
            for (key, value) in map {
                merged_map.insert(key, value);
            }
        } else {
            return Err("Each JSON file must contain a JSON object".into());
        }
    }

    let merged_json = Value::Object(merged_map);
    let json_string = serde_json::to_string_pretty(&merged_json)?;

    fs::write(output_path, json_string)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_files() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();

        fs::write(file1.path(), r#"{"a": 1, "b": 2}"#).unwrap();
        fs::write(file2.path(), r#"{"c": 3, "d": 4}"#).unwrap();

        let paths = vec![
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ];

        merge_json_files(&paths, output_file.path().to_str().unwrap()).unwrap();

        let content = fs::read_to_string(output_file.path()).unwrap();
        let parsed: Value = serde_json::from_str(&content).unwrap();

        assert_eq!(parsed["a"], 1);
        assert_eq!(parsed["b"], 2);
        assert_eq!(parsed["c"], 3);
        assert_eq!(parsed["d"], 4);
    }
}