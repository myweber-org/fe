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

        if let serde_json::Value::Object(map) = json_value {
            for (key, value) in map {
                merged_map.insert(key, value);
            }
        }
    }

    Ok(serde_json::Value::Object(merged_map))
}use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub fn merge_json_files(file_paths: &[&str], output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged: HashMap<String, Value> = HashMap::new();

    for file_path in file_paths {
        let content = fs::read_to_string(file_path)?;
        let data: Value = serde_json::from_str(&content)?;

        if let Value::Object(map) = data {
            for (key, value) in map {
                merged.insert(key, value);
            }
        } else {
            return Err("Each JSON file must contain an object at the top level".into());
        }
    }

    let merged_value = Value::Object(
        merged
            .into_iter()
            .map(|(k, v)| (k, v))
            .collect()
    );

    let output = serde_json::to_string_pretty(&merged_value)?;
    fs::write(output_path, output)?;

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
        let output_file = NamedTempFile::new().unwrap();

        fs::write(file1.path(), r#"{"a": 1, "b": 2}"#).unwrap();
        fs::write(file2.path(), r#"{"c": 3, "d": 4}"#).unwrap();

        merge_json_files(
            &[file1.path().to_str().unwrap(), file2.path().to_str().unwrap()],
            output_file.path().to_str().unwrap()
        ).unwrap();

        let content = fs::read_to_string(output_file.path()).unwrap();
        let parsed: Value = serde_json::from_str(&content).unwrap();

        assert_eq!(parsed["a"], 1);
        assert_eq!(parsed["b"], 2);
        assert_eq!(parsed["c"], 3);
        assert_eq!(parsed["d"], 4);
    }
}