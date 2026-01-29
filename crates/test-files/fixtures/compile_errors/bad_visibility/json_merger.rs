
use serde_json::{Value, Map};
use std::fs;
use std::path::Path;
use std::collections::HashSet;

pub fn merge_json_files(file_paths: &[&str], output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged_map = Map::new();
    let mut processed_keys = HashSet::new();
    let mut conflict_log = Vec::new();

    for file_path in file_paths {
        let content = fs::read_to_string(file_path)?;
        let json_value: Value = serde_json::from_str(&content)?;

        if let Value::Object(obj) = json_value {
            for (key, value) in obj {
                if processed_keys.contains(&key) {
                    conflict_log.push(format!("Conflict detected for key '{}' in file '{}'", key, file_path));
                    continue;
                }
                merged_map.insert(key.clone(), value);
                processed_keys.insert(key);
            }
        }
    }

    let merged_json = Value::Object(merged_map);
    let pretty_json = serde_json::to_string_pretty(&merged_json)?;

    fs::write(output_path, pretty_json)?;

    if !conflict_log.is_empty() {
        let log_path = Path::new(output_path).with_extension("conflicts.log");
        fs::write(log_path, conflict_log.join("\n"))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_files() {
        let file1_content = r#"{"name": "Alice", "age": 30}"#;
        let file2_content = r#"{"city": "Berlin", "country": "Germany"}"#;

        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();

        fs::write(file1.path(), file1_content).unwrap();
        fs::write(file2.path(), file2_content).unwrap();

        let paths = vec![
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ];

        merge_json_files(&paths, output_file.path().to_str().unwrap()).unwrap();

        let merged_content = fs::read_to_string(output_file.path()).unwrap();
        let parsed: Value = serde_json::from_str(&merged_content).unwrap();

        assert!(parsed.get("name").is_some());
        assert!(parsed.get("city").is_some());
        assert_eq!(parsed.get("age").unwrap().as_i64(), Some(30));
    }
}