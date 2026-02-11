use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, Read};
use std::path::Path;

use serde_json::{Value, json};

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P], output_path: P) -> Result<(), String> {
    let mut merged_array = Vec::new();

    for path in paths {
        let file = File::open(path).map_err(|e| format!("Failed to open {}: {}", path.as_ref().display(), e))?;
        let mut reader = BufReader::new(file);
        let mut contents = String::new();
        reader.read_to_string(&mut contents).map_err(|e| format!("Failed to read {}: {}", path.as_ref().display(), e))?;

        let json_value: Value = serde_json::from_str(&contents)
            .map_err(|e| format!("Failed to parse JSON from {}: {}", path.as_ref().display(), e))?;

        if let Value::Array(arr) = json_value {
            merged_array.extend(arr);
        } else {
            merged_array.push(json_value);
        }
    }

    let merged_json = json!(merged_array);
    let json_string = serde_json::to_string_pretty(&merged_json)
        .map_err(|e| format!("Failed to serialize merged JSON: {}", e))?;

    fs::write(&output_path, json_string)
        .map_err(|e| format!("Failed to write output file {}: {}", output_path.as_ref().display(), e))?;

    Ok(())
}

pub fn merge_json_with_deduplication<P: AsRef<Path>>(paths: &[P], output_path: P, key_field: &str) -> Result<usize, String> {
    let mut unique_map: HashMap<String, Value> = HashMap::new();
    let mut total_processed = 0;

    for path in paths {
        let file = File::open(path).map_err(|e| format!("Failed to open {}: {}", path.as_ref().display(), e))?;
        let mut reader = BufReader::new(file);
        let mut contents = String::new();
        reader.read_to_string(&mut contents).map_err(|e| format!("Failed to read {}: {}", path.as_ref().display(), e))?;

        let json_value: Value = serde_json::from_str(&contents)
            .map_err(|e| format!("Failed to parse JSON from {}: {}", path.as_ref().display(), e))?;

        let items = match json_value {
            Value::Array(arr) => arr,
            _ => vec![json_value],
        };

        for item in items {
            total_processed += 1;
            if let Some(obj) = item.as_object() {
                if let Some(key_value) = obj.get(key_field) {
                    if let Some(key_str) = key_value.as_str() {
                        unique_map.insert(key_str.to_string(), item);
                    }
                }
            }
        }
    }

    let unique_values: Vec<Value> = unique_map.into_values().collect();
    let merged_json = json!(unique_values);
    let json_string = serde_json::to_string_pretty(&merged_json)
        .map_err(|e| format!("Failed to serialize deduplicated JSON: {}", e))?;

    fs::write(&output_path, json_string)
        .map_err(|e| format!("Failed to write output file {}: {}", output_path.as_ref().display(), e))?;

    Ok(total_processed - unique_values.len())
}