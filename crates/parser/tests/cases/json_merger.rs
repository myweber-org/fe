
use serde_json::{json, Value};
use std::collections::HashSet;
use std::fs;
use std::path::Path;

pub fn merge_json_files(file_paths: &[&str], output_path: &str) -> Result<(), String> {
    let mut merged_array = Vec::new();
    let mut seen_ids = HashSet::new();

    for file_path in file_paths {
        let path = Path::new(file_path);
        if !path.exists() {
            return Err(format!("File not found: {}", file_path));
        }

        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read {}: {}", file_path, e))?;

        let json_value: Value = serde_json::from_str(&content)
            .map_err(|e| format!("Invalid JSON in {}: {}", file_path, e))?;

        match json_value {
            Value::Array(arr) => {
                for item in arr {
                    if let Some(id) = item.get("id").and_then(Value::as_str) {
                        if !seen_ids.contains(id) {
                            seen_ids.insert(id.to_string());
                            merged_array.push(item);
                        }
                    } else {
                        merged_array.push(item);
                    }
                }
            }
            Value::Object(obj) => merged_array.push(json!(obj)),
            _ => return Err(format!("JSON root must be array or object in {}", file_path)),
        }
    }

    let output_json = json!(merged_array);
    let pretty_json = serde_json::to_string_pretty(&output_json)
        .map_err(|e| format!("Failed to serialize output JSON: {}", e))?;

    fs::write(output_path, pretty_json)
        .map_err(|e| format!("Failed to write output file: {}", e))?;

    println!("Successfully merged {} files into {}", file_paths.len(), output_path);
    Ok(())
}