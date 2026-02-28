
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
use serde_json::{Value, Map};
use std::fs;
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P]) -> Result<Value, Box<dyn std::error::Error>> {
    let mut merged = Map::new();

    for path in paths {
        let content = fs::read_to_string(path)?;
        let json: Value = serde_json::from_str(&content)?;

        if let Value::Object(obj) = json {
            for (key, value) in obj {
                if merged.contains_key(&key) {
                    let existing = merged.get(&key).unwrap();
                    if existing != &value {
                        merged.insert(key, Value::String(format!("CONFLICT: {:?} vs {:?}", existing, value)));
                    } else {
                        merged.insert(key, value);
                    }
                } else {
                    merged.insert(key, value);
                }
            }
        } else {
            return Err("Top-level JSON must be an object".into());
        }
    }

    Ok(Value::Object(merged))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json() -> Result<(), Box<dyn std::error::Error>> {
        let file1 = NamedTempFile::new()?;
        let file2 = NamedTempFile::new()?;

        fs::write(&file1, r#"{"a": 1, "b": "test"}"#)?;
        fs::write(&file2, r#"{"b": "different", "c": true}"#)?;

        let result = merge_json_files(&[file1.path(), file2.path()])?;
        
        assert_eq!(result["a"], json!(1));
        assert!(result["b"].as_str().unwrap().contains("CONFLICT"));
        assert_eq!(result["c"], json!(true));

        Ok(())
    }
}use std::collections::HashMap;
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
        let json2 = r#"{"city": "London", "active": true}"#;

        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();
        file1.write_all(json1.as_bytes()).unwrap();
        file2.write_all(json2.as_bytes()).unwrap();

        let paths = vec![
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ];

        let result = merge_json_files(&paths).unwrap();
        let expected = serde_json::json!({
            "name": "Alice",
            "age": 30,
            "city": "London",
            "active": true
        });

        assert_eq!(result, expected);
    }
}