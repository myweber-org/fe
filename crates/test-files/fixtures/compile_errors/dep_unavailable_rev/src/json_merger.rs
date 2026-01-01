
use serde_json::{Map, Value};
use std::collections::HashSet;
use std::fs;
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P], output_path: P) -> Result<(), String> {
    if paths.is_empty() {
        return Err("No input files provided".to_string());
    }

    let mut merged_map = Map::new();
    let mut conflict_log = Vec::new();

    for (idx, path) in paths.iter().enumerate() {
        let content = fs::read_to_string(path).map_err(|e| format!("Failed to read {}: {}", path.as_ref().display(), e))?;
        let json_value: Value = serde_json::from_str(&content).map_err(|e| format!("Invalid JSON in {}: {}", path.as_ref().display(), e))?;

        if let Value::Object(map) = json_value {
            for (key, value) in map {
                if merged_map.contains_key(&key) {
                    let existing_value = merged_map.get(&key).unwrap();
                    if existing_value != &value {
                        conflict_log.push(format!(
                            "Conflict at key '{}': file {} has {:?}, previous files had {:?}",
                            key,
                            idx + 1,
                            value,
                            existing_value
                        ));
                        merged_map.insert(key + "_conflict", value);
                    }
                } else {
                    merged_map.insert(key, value);
                }
            }
        } else {
            return Err(format!("Top-level JSON must be an object in {}", path.as_ref().display()));
        }
    }

    let merged_value = Value::Object(merged_map);
    let pretty_json = serde_json::to_string_pretty(&merged_value).map_err(|e| e.to_string())?;
    fs::write(&output_path, pretty_json).map_err(|e| format!("Failed to write output: {}", e))?;

    if !conflict_log.is_empty() {
        let log_path = output_path.as_ref().with_extension("conflicts.log");
        fs::write(log_path, conflict_log.join("\n")).map_err(|e| format!("Failed to write conflict log: {}", e))?;
        return Err("Merged with conflicts. Check .conflicts.log file.".to_string());
    }

    Ok(())
}

pub fn find_unique_keys<P: AsRef<Path>>(paths: &[P]) -> Result<Vec<HashSet<String>>, String> {
    let mut key_sets = Vec::new();

    for path in paths {
        let content = fs::read_to_string(path).map_err(|e| format!("Failed to read {}: {}", path.as_ref().display(), e))?;
        let json_value: Value = serde_json::from_str(&content).map_err(|e| format!("Invalid JSON in {}: {}", path.as_ref().display(), e))?;

        if let Value::Object(map) = json_value {
            let keys: HashSet<String> = map.keys().cloned().collect();
            key_sets.push(keys);
        } else {
            return Err(format!("Top-level JSON must be an object in {}", path.as_ref().display()));
        }
    }

    Ok(key_sets)
}