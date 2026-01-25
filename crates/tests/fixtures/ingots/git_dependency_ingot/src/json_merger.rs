
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use serde_json::{Value, Map};

pub fn merge_json_files(file_paths: &[&str]) -> Result<Value, Box<dyn std::error::Error>> {
    let mut merged_map = Map::new();

    for file_path in file_paths {
        if !Path::new(file_path).exists() {
            return Err(format!("File not found: {}", file_path).into());
        }

        let content = fs::read_to_string(file_path)?;
        let json_value: Value = serde_json::from_str(&content)?;

        if let Value::Object(map) = json_value {
            for (key, value) in map {
                if merged_map.contains_key(&key) {
                    eprintln!("Warning: Key '{}' already exists, overwriting.", key);
                }
                merged_map.insert(key, value);
            }
        } else {
            return Err("Top-level JSON must be an object".into());
        }
    }

    Ok(Value::Object(merged_map))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_files() {
        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();

        writeln!(file1, r#"{"name": "Alice", "age": 30}"#).unwrap();
        writeln!(file2, r#"{"city": "Berlin", "age": 31}"#).unwrap();

        let result = merge_json_files(&[
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ]).unwrap();

        let expected: Value = serde_json::from_str(r#"{"name": "Alice", "age": 31, "city": "Berlin"}"#).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_file_not_found() {
        let result = merge_json_files(&["nonexistent.json"]);
        assert!(result.is_err());
    }
}