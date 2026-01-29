
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

pub fn merge_json_files(file_paths: &[&str]) -> Result<Value, Box<dyn std::error::Error>> {
    let mut merged_array = Vec::new();
    let mut seen_keys = HashMap::new();

    for path_str in file_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            eprintln!("Warning: File {} not found, skipping.", path_str);
            continue;
        }

        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        let mut content = String::new();
        reader.read_to_string(&mut content)?;

        let parsed: Value = serde_json::from_str(&content)?;

        match parsed {
            Value::Array(arr) => {
                for item in arr {
                    if let Some(obj) = item.as_object() {
                        if let Some(id) = obj.get("id").and_then(|v| v.as_str()) {
                            if !seen_keys.contains_key(id) {
                                seen_keys.insert(id.to_string(), true);
                                merged_array.push(item);
                            }
                        } else {
                            merged_array.push(item);
                        }
                    } else {
                        merged_array.push(item);
                    }
                }
            }
            Value::Object(_) => merged_array.push(parsed),
            _ => eprintln!("Warning: File {} does not contain JSON object or array.", path_str),
        }
    }

    Ok(json!(merged_array))
}

pub fn write_merged_json(output_path: &str, value: &Value) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::create(output_path)?;
    serde_json::to_writer_pretty(file, value)?;
    Ok(())
}