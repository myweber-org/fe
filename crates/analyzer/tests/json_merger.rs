
use serde_json::{Value, Map};
use std::fs;
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P]) -> Result<Value, Box<dyn std::error::Error>> {
    let mut merged_map = Map::new();

    for path in paths {
        let content = fs::read_to_string(path)?;
        let json_value: Value = serde_json::from_str(&content)?;

        if let Value::Object(map) = json_value {
            for (key, value) in map {
                merge_value(&mut merged_map, key, value);
            }
        }
    }

    Ok(Value::Object(merged_map))
}

fn merge_value(map: &mut Map<String, Value>, key: String, new_value: Value) {
    match map.get_mut(&key) {
        Some(existing_value) => {
            if let (Value::Object(existing_obj), Value::Object(new_obj)) = (existing_value, &new_value) {
                let mut existing_obj = existing_obj.clone();
                for (nested_key, nested_value) in new_obj {
                    merge_value(&mut existing_obj, nested_key.clone(), nested_value.clone());
                }
                map.insert(key, Value::Object(existing_obj));
            } else if existing_value != &new_value {
                let conflict_array = vec![existing_value.clone(), new_value];
                map.insert(key, Value::Array(conflict_array));
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
    fn test_merge_json() -> Result<(), Box<dyn std::error::Error>> {
        let file1 = NamedTempFile::new()?;
        let file2 = NamedTempFile::new()?;

        fs::write(&file1, r#"{"name": "Alice", "age": 30}"#)?;
        fs::write(&file2, r#"{"name": "Bob", "city": "London"}"#)?;

        let result = merge_json_files(&[file1.path(), file2.path()])?;
        
        assert_eq!(result["name"], json!(["Alice", "Bob"]));
        assert_eq!(result["age"], json!(30));
        assert_eq!(result["city"], json!("London"));

        Ok(())
    }
}use serde_json::{Map, Value};
use std::collections::HashSet;

pub fn merge_json(base: &mut Value, update: &Value, deep: bool) {
    match (base, update) {
        (Value::Object(base_map), Value::Object(update_map)) => {
            for (key, update_value) in update_map {
                if deep {
                    if let Some(base_value) = base_map.get_mut(key) {
                        merge_json(base_value, update_value, deep);
                    } else {
                        base_map.insert(key.clone(), update_value.clone());
                    }
                } else {
                    base_map.insert(key.clone(), update_value.clone());
                }
            }
        }
        (base, update) => *base = update.clone(),
    }
}

pub fn merge_json_with_conflict_resolution(
    base: &mut Value,
    update: &Value,
    conflict_strategy: ConflictStrategy,
) -> HashSet<String> {
    let mut conflicts = HashSet::new();
    
    if let (Value::Object(base_map), Value::Object(update_map)) = (base, update) {
        for (key, update_value) in update_map {
            if let Some(base_value) = base_map.get_mut(key) {
                if base_value != update_value {
                    conflicts.insert(key.clone());
                    match conflict_strategy {
                        ConflictStrategy::PreferUpdate => {
                            *base_value = update_value.clone();
                        }
                        ConflictStrategy::PreferBase => {}
                        ConflictStrategy::MergeArrays => {
                            if let (Value::Array(base_arr), Value::Array(update_arr)) = (base_value, update_value) {
                                let mut merged = base_arr.clone();
                                merged.extend(update_arr.clone());
                                *base_value = Value::Array(merged);
                            } else {
                                *base_value = update_value.clone();
                            }
                        }
                    }
                }
            } else {
                base_map.insert(key.clone(), update_value.clone());
            }
        }
    }
    
    conflicts
}

#[derive(Debug, Clone, Copy)]
pub enum ConflictStrategy {
    PreferBase,
    PreferUpdate,
    MergeArrays,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_shallow_merge() {
        let mut base = json!({"a": 1, "b": 2});
        let update = json!({"b": 3, "c": 4});
        
        merge_json(&mut base, &update, false);
        
        assert_eq!(base, json!({"a": 1, "b": 3, "c": 4}));
    }

    #[test]
    fn test_deep_merge() {
        let mut base = json!({"a": {"x": 1}, "b": 2});
        let update = json!({"a": {"y": 3}, "c": 4});
        
        merge_json(&mut base, &update, true);
        
        assert_eq!(base, json!({"a": {"x": 1, "y": 3}, "b": 2, "c": 4}));
    }

    #[test]
    fn test_conflict_detection() {
        let mut base = json!({"a": 1, "b": 2});
        let update = json!({"a": 99, "c": 3});
        
        let conflicts = merge_json_with_conflict_resolution(
            &mut base,
            &update,
            ConflictStrategy::PreferUpdate
        );
        
        assert!(conflicts.contains("a"));
        assert!(!conflicts.contains("c"));
        assert_eq!(base["a"], 99);
    }
}