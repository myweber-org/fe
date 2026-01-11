use serde_json::{Map, Value};
use std::collections::HashMap;
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

pub fn merge_json_with_strategy(
    file_paths: &[&str],
    strategy: MergeStrategy,
) -> Result<Value, Box<dyn std::error::Error>> {
    let mut accumulator: HashMap<String, Value> = HashMap::new();

    for path_str in file_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            continue;
        }

        let content = fs::read_to_string(path)?;
        let json_value: Value = serde_json::from_str(&content)?;

        if let Value::Object(obj) = json_value {
            for (key, value) in obj {
                match strategy {
                    MergeStrategy::Overwrite => {
                        accumulator.insert(key, value);
                    }
                    MergeStrategy::CombineArrays => {
                        if let Some(existing) = accumulator.get(&key) {
                            if existing.is_array() && value.is_array() {
                                let mut combined = existing.as_array().unwrap().clone();
                                combined.extend(value.as_array().unwrap().clone());
                                accumulator.insert(key, Value::Array(combined));
                            } else {
                                accumulator.insert(key, value);
                            }
                        } else {
                            accumulator.insert(key, value);
                        }
                    }
                    MergeStrategy::SkipDuplicate => {
                        if !accumulator.contains_key(&key) {
                            accumulator.insert(key, value);
                        }
                    }
                }
            }
        }
    }

    let map: Map<String, Value> = accumulator.into_iter().collect();
    Ok(Value::Object(map))
}

#[derive(Clone, Copy)]
pub enum MergeStrategy {
    Overwrite,
    CombineArrays,
    SkipDuplicate,
}