use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

pub fn merge_json_files(file_paths: &[&str]) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let mut merged_map = HashMap::new();

    for path_str in file_paths {
        let path = Path::new(path_str);
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        let mut contents = String::new();
        reader.read_to_string(&mut contents)?;

        let json_value: serde_json::Value = serde_json::from_str(&contents)?;

        if let serde_json::Value::Object(map) = json_value {
            for (key, value) in map {
                merged_map.insert(key, value);
            }
        } else {
            return Err("Each JSON file must contain a JSON object".into());
        }
    }

    Ok(serde_json::Value::Object(merged_map))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_files() {
        let json1 = r#"{"name": "Alice", "age": 30}"#;
        let json2 = r#"{"city": "Berlin", "country": "Germany"}"#;

        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();

        file1.write_all(json1.as_bytes()).unwrap();
        file2.write_all(json2.as_bytes()).unwrap();

        let paths = vec![
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ];

        let result = merge_json_files(&paths).unwrap();
        let result_obj = result.as_object().unwrap();

        assert_eq!(result_obj.get("name").unwrap(), "Alice");
        assert_eq!(result_obj.get("age").unwrap(), 30);
        assert_eq!(result_obj.get("city").unwrap(), "Berlin");
        assert_eq!(result_obj.get("country").unwrap(), "Germany");
        assert_eq!(result_obj.len(), 4);
    }
}
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read, Write};
use std::path::Path;

use serde_json::{Map, Value};

pub fn merge_json_files(file_paths: &[&str]) -> Result<Value, String> {
    let mut merged_map = Map::new();

    for path_str in file_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            return Err(format!("File not found: {}", path_str));
        }

        let mut file = match File::open(path) {
            Ok(f) => f,
            Err(e) => return Err(format!("Failed to open {}: {}", path_str, e)),
        };

        let mut content = String::new();
        if let Err(e) = file.read_to_string(&mut content) {
            return Err(format!("Failed to read {}: {}", path_str, e));
        }

        let json_value: Value = match serde_json::from_str(&content) {
            Ok(v) => v,
            Err(e) => return Err(format!("Invalid JSON in {}: {}", path_str, e)),
        };

        if let Value::Object(obj) = json_value {
            for (key, value) in obj {
                if merged_map.contains_key(&key) {
                    return Err(format!("Duplicate key '{}' found in {}", key, path_str));
                }
                merged_map.insert(key, value);
            }
        } else {
            return Err(format!("Top-level JSON must be an object in {}", path_str));
        }
    }

    Ok(Value::Object(merged_map))
}

pub fn write_merged_json(output_path: &str, value: &Value) -> Result<(), String> {
    let json_string = match serde_json::to_string_pretty(value) {
        Ok(s) => s,
        Err(e) => return Err(format!("Failed to serialize JSON: {}", e)),
    };

    let mut file = match File::create(output_path) {
        Ok(f) => f,
        Err(e) => return Err(format!("Failed to create output file: {}", e)),
    };

    match file.write_all(json_string.as_bytes()) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Failed to write output file: {}", e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_files() {
        let file1_content = r#"{"name": "Alice", "age": 30}"#;
        let file2_content = r#"{"city": "London", "country": "UK"}"#;

        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();

        fs::write(file1.path(), file1_content).unwrap();
        fs::write(file2.path(), file2_content).unwrap();

        let paths = vec![
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ];

        let result = merge_json_files(&paths);
        assert!(result.is_ok());

        let merged = result.unwrap();
        let obj = merged.as_object().unwrap();

        assert_eq!(obj.get("name").unwrap().as_str().unwrap(), "Alice");
        assert_eq!(obj.get("age").unwrap().as_u64().unwrap(), 30);
        assert_eq!(obj.get("city").unwrap().as_str().unwrap(), "London");
        assert_eq!(obj.get("country").unwrap().as_str().unwrap(), "UK");
    }

    #[test]
    fn test_duplicate_key_error() {
        let file1_content = r#"{"key": "value1"}"#;
        let file2_content = r#"{"key": "value2"}"#;

        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();

        fs::write(file1.path(), file1_content).unwrap();
        fs::write(file2.path(), file2_content).unwrap();

        let paths = vec![
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ];

        let result = merge_json_files(&paths);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Duplicate key"));
    }
}