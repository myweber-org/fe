use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, Read};
use std::path::Path;

type JsonValue = serde_json::Value;

pub fn merge_json_files(file_paths: &[impl AsRef<Path>]) -> Result<JsonValue, Box<dyn std::error::Error>> {
    let mut merged_array = Vec::new();

    for path in file_paths {
        let file = File::open(path.as_ref())?;
        let mut reader = BufReader::new(file);
        let mut contents = String::new();
        reader.read_to_string(&mut contents)?;

        let json_value: JsonValue = serde_json::from_str(&contents)?;
        
        match json_value {
            JsonValue::Array(arr) => {
                merged_array.extend(arr);
            }
            _ => {
                merged_array.push(json_value);
            }
        }
    }

    Ok(JsonValue::Array(merged_array))
}

pub fn merge_json_with_key(
    file_paths: &[impl AsRef<Path>],
    key: &str,
) -> Result<JsonValue, Box<dyn std::error::Error>> {
    let mut merged_map = HashMap::new();

    for path in file_paths {
        let contents = fs::read_to_string(path.as_ref())?;
        let json_value: JsonValue = serde_json::from_str(&contents)?;

        if let JsonValue::Object(obj) = json_value {
            if let Some(value) = obj.get(key) {
                merged_map.insert(key.to_string(), value.clone());
            }
        }
    }

    Ok(JsonValue::Object(serde_json::Map::from_iter(merged_map)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_files() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();

        fs::write(&file1, r#"[{"id": 1}, {"id": 2}]"#).unwrap();
        fs::write(&file2, r#"{"id": 3}"#).unwrap();

        let result = merge_json_files(&[file1.path(), file2.path()]).unwrap();
        assert!(result.is_array());
        assert_eq!(result.as_array().unwrap().len(), 3);
    }

    #[test]
    fn test_merge_json_with_key() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();

        fs::write(&file1, r#"{"user": "alice", "age": 30}"#).unwrap();
        fs::write(&file2, r#"{"user": "bob", "age": 25}"#).unwrap();

        let result = merge_json_with_key(&[file1.path(), file2.path()], "user").unwrap();
        assert!(result.is_object());
        let obj = result.as_object().unwrap();
        assert!(obj.contains_key("user"));
    }
}
use serde_json::{Map, Value};
use std::env;
use std::fs;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <output_file.json> <input1.json> [input2.json ...]", args[0]);
        process::exit(1);
    }

    let output_path = &args[1];
    let input_paths = &args[2..];

    let mut merged_map = Map::new();

    for input_path in input_paths {
        let content = match fs::read_to_string(input_path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Failed to read {}: {}", input_path, e);
                process::exit(1);
            }
        };

        let json_value: Value = match serde_json::from_str(&content) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("Failed to parse JSON from {}: {}", input_path, e);
                process::exit(1);
            }
        };

        if let Value::Object(map) = json_value {
            for (key, value) in map {
                merged_map.insert(key, value);
            }
        } else {
            eprintln!("Top-level JSON in {} is not an object", input_path);
            process::exit(1);
        }
    }

    let merged_value = Value::Object(merged_map);
    let json_string = match serde_json::to_string_pretty(&merged_value) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to serialize merged JSON: {}", e);
            process::exit(1);
        }
    };

    if let Err(e) = fs::write(output_path, json_string) {
        eprintln!("Failed to write to {}: {}", output_path, e);
        process::exit(1);
    }

    println!("Successfully merged {} files into {}", input_paths.len(), output_path);
}