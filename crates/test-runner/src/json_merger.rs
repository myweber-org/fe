use serde_json::{Map, Value};
use std::fs;
use std::path::Path;

pub fn merge_json_files(file_paths: &[&str]) -> Result<Value, Box<dyn std::error::Error>> {
    let mut merged_map = Map::new();

    for path_str in file_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            return Err(format!("File not found: {}", path_str).into());
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
            return Err("Each JSON file must contain a JSON object at its root.".into());
        }
    }

    Ok(Value::Object(merged_map))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_files() {
        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();

        writeln!(file1, r#"{{ "name": "Alice", "age": 30 }}"#).unwrap();
        writeln!(file2, r#"{{ "city": "Berlin", "active": true }}"#).unwrap();

        let paths = [
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ];

        let result = merge_json_files(&paths).unwrap();
        let expected = json!({
            "name": "Alice",
            "age": 30,
            "city": "Berlin",
            "active": true
        });

        assert_eq!(result, expected);
    }

    #[test]
    fn test_overwrite_keys() {
        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();

        writeln!(file1, r#"{{ "id": 1, "value": "old" }}"#).unwrap();
        writeln!(file2, r#"{{ "id": 2, "extra": "data" }}"#).unwrap();

        let paths = [
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ];

        let result = merge_json_files(&paths).unwrap();
        assert_eq!(result["id"], json!(2));
        assert_eq!(result["value"], json!("old"));
        assert_eq!(result["extra"], json!("data"));
    }
}
use serde_json::{Map, Value};
use std::env;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <output_file> <input_file1> [input_file2 ...]", args[0]);
        std::process::exit(1);
    }

    let output_path = &args[1];
    let input_paths = &args[2..];

    let mut merged_map = Map::new();

    for (index, input_path) in input_paths.iter().enumerate() {
        let content = fs::read_to_string(input_path)?;
        let parsed: Value = serde_json::from_str(&content)?;

        if let Value::Object(map) = parsed {
            for (key, value) in map {
                let unique_key = if merged_map.contains_key(&key) {
                    format!("{}_{}", key, index)
                } else {
                    key
                };
                merged_map.insert(unique_key, value);
            }
        } else {
            eprintln!("Warning: File '{}' does not contain a JSON object at root. Skipping.", input_path);
        }
    }

    let merged_value = Value::Object(merged_map);
    let output_content = serde_json::to_string_pretty(&merged_value)?;
    fs::write(output_path, output_content)?;

    println!("Successfully merged {} files into '{}'.", input_paths.len(), output_path);
    Ok(())
}