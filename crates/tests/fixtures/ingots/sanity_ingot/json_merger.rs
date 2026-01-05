
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub fn merge_json_files(file_paths: &[&str]) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let mut merged_map = HashMap::new();

    for path_str in file_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            return Err(format!("File not found: {}", path_str).into());
        }

        let content = fs::read_to_string(path)?;
        let json_value: serde_json::Value = serde_json::from_str(&content)?;

        if let serde_json::Value::Object(map) = json_value {
            for (key, value) in map {
                if merged_map.contains_key(&key) {
                    eprintln!("Warning: Key '{}' already exists, overwriting with value from {}", key, path_str);
                }
                merged_map.insert(key, value);
            }
        } else {
            return Err("Top-level JSON must be an object".into());
        }
    }

    Ok(serde_json::Value::Object(merged_map))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_files() {
        let file1_content = json!({"a": 1, "b": "test"});
        let file2_content = json!({"c": true, "d": [1, 2, 3]});

        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();

        fs::write(file1.path(), serde_json::to_string(&file1_content).unwrap()).unwrap();
        fs::write(file2.path(), serde_json::to_string(&file2_content).unwrap()).unwrap();

        let result = merge_json_files(&[file1.path().to_str().unwrap(), file2.path().to_str().unwrap()]);
        assert!(result.is_ok());

        let merged = result.unwrap();
        assert_eq!(merged["a"], 1);
        assert_eq!(merged["b"], "test");
        assert_eq!(merged["c"], true);
        assert_eq!(merged["d"], json!([1, 2, 3]));
    }

    #[test]
    fn test_merge_with_overwrite() {
        let file1_content = json!({"key": "original"});
        let file2_content = json!({"key": "overwritten"});

        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();

        fs::write(file1.path(), serde_json::to_string(&file1_content).unwrap()).unwrap();
        fs::write(file2.path(), serde_json::to_string(&file2_content).unwrap()).unwrap();

        let result = merge_json_files(&[file1.path().to_str().unwrap(), file2.path().to_str().unwrap()]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap()["key"], "overwritten");
    }
}