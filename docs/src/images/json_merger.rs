
use serde_json::{Value, Map};
use std::fs;
use std::path::Path;
use std::collections::HashSet;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P], output_path: P) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged = Map::new();
    let mut processed_keys = HashSet::new();
    let mut conflict_log = Vec::new();

    for path in paths {
        let content = fs::read_to_string(path)?;
        let json: Value = serde_json::from_str(&content)?;
        
        if let Value::Object(obj) = json {
            for (key, value) in obj {
                if processed_keys.contains(&key) {
                    conflict_log.push(format!("Conflict detected for key '{}' in file: {:?}", key, path.as_ref()));
                    continue;
                }
                merged.insert(key.clone(), value);
                processed_keys.insert(key);
            }
        }
    }

    let result = Value::Object(merged);
    let serialized = serde_json::to_string_pretty(&result)?;
    fs::write(output_path, serialized)?;

    if !conflict_log.is_empty() {
        eprintln!("Merged with conflicts:");
        for log in conflict_log {
            eprintln!("  {}", log);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_basic_merge() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        let output = NamedTempFile::new().unwrap();

        fs::write(&file1, r#"{"a": 1, "b": 2}"#).unwrap();
        fs::write(&file2, r#"{"c": 3, "d": 4}"#).unwrap();

        merge_json_files(&[&file1, &file2], &output).unwrap();
        
        let content = fs::read_to_string(output).unwrap();
        let parsed: Value = serde_json::from_str(&content).unwrap();
        
        assert_eq!(parsed["a"], 1);
        assert_eq!(parsed["b"], 2);
        assert_eq!(parsed["c"], 3);
        assert_eq!(parsed["d"], 4);
    }

    #[test]
    fn test_conflict_handling() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        let output = NamedTempFile::new().unwrap();

        fs::write(&file1, r#"{"a": 1, "b": 2}"#).unwrap();
        fs::write(&file2, r#"{"a": 99, "c": 3}"#).unwrap();

        merge_json_files(&[&file1, &file2], &output).unwrap();
        
        let content = fs::read_to_string(output).unwrap();
        let parsed: Value = serde_json::from_str(&content).unwrap();
        
        assert_eq!(parsed["a"], 1);
        assert_eq!(parsed["b"], 2);
        assert_eq!(parsed["c"], 3);
    }
}