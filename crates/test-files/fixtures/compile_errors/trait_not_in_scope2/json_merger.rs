use serde_json::{Map, Value};
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

        if let Value::Object(obj) = json_value {
            for (key, value) in obj {
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

        let obj = result.as_object().unwrap();
        assert_eq!(obj.get("name").unwrap(), "Alice");
        assert_eq!(obj.get("age").unwrap(), 30);
        assert_eq!(obj.get("city").unwrap(), "Berlin");
        assert_eq!(obj.get("active").unwrap(), true);
    }
}
use serde_json::{Value, Map};
use std::collections::HashSet;

pub fn merge_json(base: &mut Value, update: &Value, strategy: MergeStrategy) -> Result<(), String> {
    match (base, update) {
        (Value::Object(base_map), Value::Object(update_map)) => {
            merge_objects(base_map, update_map, strategy)
        }
        (Value::Array(base_arr), Value::Array(update_arr)) => {
            merge_arrays(base_arr, update_arr, strategy)
        }
        (base_val, update_val) if base_val.is_null() => {
            *base = update_val.clone();
            Ok(())
        }
        (base_val, update_val) if base_val != update_val => {
            match strategy {
                MergeStrategy::PreferUpdate => {
                    *base = update_val.clone();
                    Ok(())
                }
                MergeStrategy::PreferBase => Ok(()),
                MergeStrategy::FailOnConflict => {
                    Err("Conflict between primitive values".to_string())
                }
            }
        }
        _ => Ok(()),
    }
}

fn merge_objects(
    base: &mut Map<String, Value>,
    update: &Map<String, Value>,
    strategy: MergeStrategy,
) -> Result<(), String> {
    let base_keys: HashSet<_> = base.keys().collect();
    let update_keys: HashSet<_> = update.keys().collect();
    let conflicts: Vec<_> = base_keys.intersection(&update_keys)
        .filter(|&&key| base[key] != update[key])
        .collect();

    if !conflicts.is_empty() && strategy == MergeStrategy::FailOnConflict {
        return Err(format!("Conflicts in keys: {:?}", conflicts));
    }

    for (key, update_value) in update {
        if let Some(base_value) = base.get_mut(key) {
            if *base_value != *update_value {
                match strategy {
                    MergeStrategy::PreferUpdate => {
                        merge_json(base_value, update_value, strategy)?;
                    }
                    MergeStrategy::PreferBase => {}
                    MergeStrategy::FailOnConflict => {
                        return Err(format!("Conflict in key '{}'", key));
                    }
                }
            }
        } else {
            base.insert(key.clone(), update_value.clone());
        }
    }
    Ok(())
}

fn merge_arrays(
    base: &mut Vec<Value>,
    update: &Vec<Value>,
    strategy: MergeStrategy,
) -> Result<(), String> {
    match strategy {
        MergeStrategy::PreferUpdate => {
            base.extend_from_slice(update);
            Ok(())
        }
        MergeStrategy::PreferBase => Ok(()),
        MergeStrategy::FailOnConflict => {
            if base != update {
                Err("Arrays differ and strategy is FailOnConflict".to_string())
            } else {
                Ok(())
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MergeStrategy {
    PreferBase,
    PreferUpdate,
    FailOnConflict,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_merge_objects_prefer_update() {
        let mut base = json!({"a": 1, "b": {"c": 2}});
        let update = json!({"b": {"c": 3, "d": 4}, "e": 5});
        
        merge_json(&mut base, &update, MergeStrategy::PreferUpdate).unwrap();
        
        assert_eq!(base["a"], 1);
        assert_eq!(base["b"]["c"], 3);
        assert_eq!(base["b"]["d"], 4);
        assert_eq!(base["e"], 5);
    }

    #[test]
    fn test_merge_arrays_prefer_update() {
        let mut base = json!([1, 2, 3]);
        let update = json!([4, 5]);
        
        merge_json(&mut base, &update, MergeStrategy::PreferUpdate).unwrap();
        
        assert_eq!(base, json!([1, 2, 3, 4, 5]));
    }
}
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

type JsonValue = serde_json::Value;

pub fn merge_json_files(file_paths: &[&str]) -> Result<JsonValue, Box<dyn std::error::Error>> {
    let mut merged_map = HashMap::new();

    for path_str in file_paths {
        let path = Path::new(path_str);
        let file_name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let json_data: JsonValue = serde_json::from_str(&contents)?;
        merged_map.insert(file_name, json_data);
    }

    Ok(serde_json::to_value(merged_map)?)
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

        writeln!(file1, r#"{{ "key1": "value1" }}"#).unwrap();
        writeln!(file2, r#"{{ "key2": 42 }}"#).unwrap();

        let paths = vec![
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ];

        let result = merge_json_files(&paths).unwrap();
        let result_obj = result.as_object().unwrap();

        assert!(result_obj.contains_key("file1"));
        assert!(result_obj.contains_key("file2"));
        assert_eq!(
            result_obj["file1"]["key1"].as_str().unwrap(),
            "value1"
        );
        assert_eq!(
            result_obj["file2"]["key2"].as_i64().unwrap(),
            42
        );
    }
}