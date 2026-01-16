use serde_json::{Map, Value};
use std::fs;
use std::path::Path;

pub fn merge_json_files(input_paths: &[&str], output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged_map = Map::new();

    for path_str in input_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            eprintln!("Warning: File {} not found, skipping.", path_str);
            continue;
        }

        let content = fs::read_to_string(path)?;
        let json_value: Value = serde_json::from_str(&content)?;

        if let Value::Object(map) = json_value {
            for (key, value) in map {
                if merged_map.contains_key(&key) {
                    eprintln!("Warning: Key '{}' already exists, overwriting.", key);
                }
                merged_map.insert(key, value);
            }
        } else {
            eprintln!("Warning: File {} does not contain a JSON object, skipping.", path_str);
        }
    }

    let merged_value = Value::Object(merged_map);
    let output_json = serde_json::to_string_pretty(&merged_value)?;
    fs::write(output_path, output_json)?;

    Ok(())
}