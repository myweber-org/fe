
use serde_json::{Value, Map};
use std::fs;
use std::path::Path;
use std::env;

fn merge_json_files(file_paths: &[String]) -> Result<Value, String> {
    let mut merged_map = Map::new();

    for file_path in file_paths {
        let content = fs::read_to_string(file_path)
            .map_err(|e| format!("Failed to read file {}: {}", file_path, e))?;
        
        let json_value: Value = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse JSON from {}: {}", file_path, e))?;

        if let Value::Object(map) = json_value {
            for (key, value) in map {
                if merged_map.contains_key(&key) {
                    return Err(format!("Duplicate key '{}' found in files", key));
                }
                merged_map.insert(key, value);
            }
        } else {
            return Err(format!("File {} does not contain a JSON object", file_path));
        }
    }

    Ok(Value::Object(merged_map))
}

fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        return Err("Usage: json_merger <file1.json> <file2.json> ...".to_string());
    }

    let file_paths = &args[1..];
    let merged = merge_json_files(file_paths)?;

    let output = serde_json::to_string_pretty(&merged)
        .map_err(|e| format!("Failed to serialize merged JSON: {}", e))?;

    println!("{}", output);
    Ok(())
}