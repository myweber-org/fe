
use serde_json::{Map, Value};
use std::collections::HashSet;
use std::fs;
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P]) -> Result<Value, String> {
    if paths.is_empty() {
        return Err("No input files provided".to_string());
    }

    let mut merged = Map::new();
    let mut processed_keys = HashSet::new();

    for path in paths {
        let content = fs::read_to_string(path).map_err(|e| {
            format!("Failed to read file {:?}: {}", path.as_ref(), e)
        })?;

        let json: Value = serde_json::from_str(&content).map_err(|e| {
            format!("Invalid JSON in file {:?}: {}", path.as_ref(), e)
        })?;

        if let Value::Object(obj) = json {
            for (key, value) in obj {
                if processed_keys.contains(&key) {
                    return Err(format!("Duplicate key '{}' found in multiple files", key));
                }
                merged.insert(key.clone(), value);
                processed_keys.insert(key);
            }
        } else {
            return Err("Top-level JSON must be an object".to_string());
        }
    }

    Ok(Value::Object(merged))
}

pub fn write_merged_json<P: AsRef<Path>>(output_path: P, value: &Value) -> Result<(), String> {
    let pretty_json = serde_json::to_string_pretty(value)
        .map_err(|e| format!("Failed to serialize JSON: {}", e))?;

    fs::write(&output_path, pretty_json)
        .map_err(|e| format!("Failed to write output file: {}", e))?;

    Ok(())
}