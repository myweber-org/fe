use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub fn merge_json_files(file_paths: &[&str]) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let mut merged_map = HashMap::new();

    for path_str in file_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            continue;
        }

        let content = fs::read_to_string(path)?;
        let json_value: serde_json::Value = serde_json::from_str(&content)?;

        if let serde_json::Value::Object(obj) = json_value {
            for (key, value) in obj {
                merged_map.insert(key, value);
            }
        }
    }

    Ok(serde_json::Value::Object(
        merged_map.into_iter().collect()
    ))
}use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

pub fn merge_json_files(file_paths: &[&str]) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let mut merged_map = HashMap::new();

    for path_str in file_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            eprintln!("Warning: File {} not found, skipping.", path_str);
            continue;
        }

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
            eprintln!("Warning: {} does not contain a JSON object, skipping.", path_str);
        }
    }

    Ok(serde_json::Value::Object(merged_map))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_merge_json_files() {
        let dir = tempdir().unwrap();
        let file1_path = dir.path().join("data1.json");
        let file2_path = dir.path().join("data2.json");

        fs::write(&file1_path, r#"{"name": "Alice", "age": 30}"#).unwrap();
        fs::write(&file2_path, r#"{"city": "Berlin", "active": true}"#).unwrap();

        let paths = vec![
            file1_path.to_str().unwrap(),
            file2_path.to_str().unwrap(),
        ];

        let result = merge_json_files(&paths).unwrap();
        let obj = result.as_object().unwrap();

        assert_eq!(obj.get("name").unwrap().as_str().unwrap(), "Alice");
        assert_eq!(obj.get("age").unwrap().as_i64().unwrap(), 30);
        assert_eq!(obj.get("city").unwrap().as_str().unwrap(), "Berlin");
        assert_eq!(obj.get("active").unwrap().as_bool().unwrap(), true);
        assert_eq!(obj.len(), 4);
    }
}