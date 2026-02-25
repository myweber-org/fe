use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

use serde_json::{Map, Value};

pub fn merge_json_files(file_paths: &[&str]) -> Result<Value, String> {
    let mut merged_map = Map::new();

    for path_str in file_paths {
        let path = Path::new(path_str);
        let file = File::open(path).map_err(|e| format!("Failed to open {}: {}", path_str, e))?;
        let mut reader = BufReader::new(file);
        let mut contents = String::new();
        reader
            .read_to_string(&mut contents)
            .map_err(|e| format!("Failed to read {}: {}", path_str, e))?;

        let json_value: Value = serde_json::from_str(&contents)
            .map_err(|e| format!("Failed to parse JSON from {}: {}", path_str, e))?;

        if let Value::Object(map) = json_value {
            for (key, value) in map {
                if merged_map.contains_key(&key) {
                    return Err(format!("Duplicate key '{}' found in {}", key, path_str));
                }
                merged_map.insert(key, value);
            }
        } else {
            return Err(format!("Top-level JSON in {} is not an object", path_str));
        }
    }

    Ok(Value::Object(merged_map))
}

pub fn merge_json_with_overwrite(file_paths: &[&str]) -> Result<Value, String> {
    let mut merged_map = HashMap::new();

    for path_str in file_paths {
        let path = Path::new(path_str);
        let file = File::open(path).map_err(|e| format!("Failed to open {}: {}", path_str, e))?;
        let mut reader = BufReader::new(file);
        let mut contents = String::new();
        reader
            .read_to_string(&mut contents)
            .map_err(|e| format!("Failed to read {}: {}", path_str, e))?;

        let json_value: Value = serde_json::from_str(&contents)
            .map_err(|e| format!("Failed to parse JSON from {}: {}", path_str, e))?;

        if let Value::Object(map) = json_value {
            for (key, value) in map {
                merged_map.insert(key, value);
            }
        } else {
            return Err(format!("Top-level JSON in {} is not an object", path_str));
        }
    }

    let final_map: Map<String, Value> = merged_map.into_iter().collect();
    Ok(Value::Object(final_map))
}