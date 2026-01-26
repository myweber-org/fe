use serde_json::{json, Value, Map};
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
            return Err("Top-level JSON is not an object".into());
        }
    }

    Ok(Value::Object(merged_map))
}

pub fn write_merged_json(output_path: &str, value: &Value) -> Result<(), Box<dyn std::error::Error>> {
    let json_string = serde_json::to_string_pretty(value)?;
    fs::write(output_path, json_string)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_files() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();

        fs::write(file1.path(), r#"{"a": 1, "b": 2}"#).unwrap();
        fs::write(file2.path(), r#"{"c": 3, "d": 4}"#).unwrap();

        let paths = &[file1.path().to_str().unwrap(), file2.path().to_str().unwrap()];
        let result = merge_json_files(paths).unwrap();

        assert_eq!(result["a"], 1);
        assert_eq!(result["b"], 2);
        assert_eq!(result["c"], 3);
        assert_eq!(result["d"], 4);
    }

    #[test]
    fn test_overwrite_keys() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();

        fs::write(file1.path(), r#"{"key": "first"}"#).unwrap();
        fs::write(file2.path(), r#"{"key": "second"}"#).unwrap();

        let paths = &[file1.path().to_str().unwrap(), file2.path().to_str().unwrap()];
        let result = merge_json_files(paths).unwrap();

        assert_eq!(result["key"], "second");
    }
}