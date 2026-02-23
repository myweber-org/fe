
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
    let mut conflict_log = Vec::new();

    for (idx, path) in paths.iter().enumerate() {
        let content = fs::read_to_string(path).map_err(|e| {
            format!("Failed to read file {}: {}", path.as_ref().display(), e)
        })?;

        let json: Value = serde_json::from_str(&content).map_err(|e| {
            format!("Invalid JSON in file {}: {}", path.as_ref().display(), e)
        })?;

        if let Value::Object(obj) = json {
            for (key, value) in obj {
                if processed_keys.contains(&key) {
                    conflict_log.push(format!(
                        "Conflict: Key '{}' already exists from previous file, skipping from file {}",
                        key, idx + 1
                    ));
                    continue;
                }
                merged.insert(key.clone(), value);
                processed_keys.insert(key);
            }
        } else {
            return Err("Top-level JSON must be an object".to_string());
        }
    }

    if !conflict_log.is_empty() {
        eprintln!("Merge conflicts detected:");
        for log in &conflict_log {
            eprintln!("  {}", log);
        }
    }

    Ok(Value::Object(merged))
}

pub fn write_merged_json<P: AsRef<Path>>(output_path: P, value: &Value) -> Result<(), String> {
    let json_str = serde_json::to_string_pretty(value)
        .map_err(|e| format!("Failed to serialize JSON: {}", e))?;

    fs::write(output_path, json_str)
        .map_err(|e| format!("Failed to write output file: {}", e))?;

    Ok(())
}