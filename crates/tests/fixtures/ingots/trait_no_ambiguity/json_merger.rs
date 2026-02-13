use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, Write};
use std::path::Path;

use serde_json::{Map, Value};

pub fn merge_json_files(input_paths: &[&str], output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged_map: Map<String, Value> = Map::new();

    for path_str in input_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            eprintln!("Warning: File {} does not exist, skipping.", path_str);
            continue;
        }

        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let json_value: Value = serde_json::from_reader(reader)?;

        if let Value::Object(map) = json_value {
            for (key, value) in map {
                if merged_map.contains_key(&key) {
                    eprintln!("Warning: Key '{}' already exists, overwriting with value from {}.", key, path_str);
                }
                merged_map.insert(key, value);
            }
        } else {
            eprintln!("Warning: File {} does not contain a JSON object at root, skipping.", path_str);
        }
    }

    let output_file = File::create(output_path)?;
    serde_json::to_writer_pretty(output_file, &Value::Object(merged_map))?;

    Ok(())
}

pub fn merge_json_with_strategy(
    input_paths: &[&str],
    output_path: &str,
    conflict_strategy: ConflictStrategy,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut accumulator: HashMap<String, Value> = HashMap::new();

    for path_str in input_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            eprintln!("Warning: File {} does not exist, skipping.", path_str);
            continue;
        }

        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let json_value: Value = serde_json::from_reader(reader)?;

        if let Value::Object(map) = json_value {
            for (key, value) in map {
                match conflict_strategy {
                    ConflictStrategy::Overwrite => {
                        accumulator.insert(key, value);
                    }
                    ConflictStrategy::Skip => {
                        accumulator.entry(key).or_insert(value);
                    }
                    ConflictStrategy::MergeObjects => {
                        if let Some(existing) = accumulator.get_mut(&key) {
                            if let (Value::Object(existing_map), Value::Object(new_map)) = (existing, &value) {
                                let mut combined = existing_map.clone();
                                for (k, v) in new_map {
                                    combined.insert(k.clone(), v.clone());
                                }
                                *existing = Value::Object(combined);
                            } else {
                                eprintln!("Warning: Key '{}' conflict, cannot merge non-objects. Overwriting.", key);
                                accumulator.insert(key, value);
                            }
                        } else {
                            accumulator.insert(key, value);
                        }
                    }
                }
            }
        } else {
            eprintln!("Warning: File {} does not contain a JSON object at root, skipping.", path_str);
        }
    }

    let output_map: Map<String, Value> = accumulator.into_iter().collect();
    let output_file = File::create(output_path)?;
    serde_json::to_writer_pretty(output_file, &Value::Object(output_map))?;

    Ok(())
}

#[derive(Debug, Clone, Copy)]
pub enum ConflictStrategy {
    Overwrite,
    Skip,
    MergeObjects,
}
use serde_json::{Map, Value};

pub fn merge_json(base: &mut Value, update: &Value, resolve_conflicts: bool) -> Result<(), String> {
    match (base, update) {
        (Value::Object(base_map), Value::Object(update_map)) => {
            for (key, update_value) in update_map {
                if let Some(base_value) = base_map.get_mut(key) {
                    if base_value == update_value {
                        continue;
                    }
                    
                    if resolve_conflicts {
                        if let (Value::Object(_), Value::Object(_)) = (base_value, update_value) {
                            merge_json(base_value, update_value, resolve_conflicts)?;
                        } else {
                            *base_value = update_value.clone();
                        }
                    } else {
                        return Err(format!("Conflict detected for key '{}'", key));
                    }
                } else {
                    base_map.insert(key.clone(), update_value.clone());
                }
            }
            Ok(())
        }
        _ => Err("Both values must be JSON objects".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_merge_without_conflicts() {
        let mut base = json!({"a": 1, "b": {"c": 2}});
        let update = json!({"b": {"d": 3}, "e": 4});
        
        assert!(merge_json(&mut base, &update, false).is_ok());
        assert_eq!(base["a"], 1);
        assert_eq!(base["b"]["c"], 2);
        assert_eq!(base["b"]["d"], 3);
        assert_eq!(base["e"], 4);
    }

    #[test]
    fn test_merge_with_conflict_resolution() {
        let mut base = json!({"a": 1, "b": {"c": 2}});
        let update = json!({"a": 99, "b": {"c": 100}});
        
        assert!(merge_json(&mut base, &update, true).is_ok());
        assert_eq!(base["a"], 99);
        assert_eq!(base["b"]["c"], 100);
    }

    #[test]
    fn test_merge_conflict_without_resolution() {
        let mut base = json!({"a": 1});
        let update = json!({"a": 2});
        
        assert!(merge_json(&mut base, &update, false).is_err());
    }
}