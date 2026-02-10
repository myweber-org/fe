
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
                    conflict_log.push(format!("Conflict detected for key '{}'", key));
                    if let Some(existing) = merged.get_mut(&key) {
                        *existing = merge_values(existing.clone(), value);
                    }
                } else {
                    merged.insert(key.clone(), value);
                    processed_keys.insert(key);
                }
            }
        }
    }

    let output_json = Value::Object(merged);
    let output_str = serde_json::to_string_pretty(&output_json)?;
    fs::write(output_path, output_str)?;

    if !conflict_log.is_empty() {
        eprintln!("Merged with conflicts:");
        for log in conflict_log {
            eprintln!("  {}", log);
        }
    }

    Ok(())
}

fn merge_values(a: Value, b: Value) -> Value {
    match (a, b) {
        (Value::Object(mut map_a), Value::Object(map_b)) => {
            for (key, value) in map_b {
                if map_a.contains_key(&key) {
                    let existing = map_a.remove(&key).unwrap();
                    map_a.insert(key, merge_values(existing, value));
                } else {
                    map_a.insert(key, value);
                }
            }
            Value::Object(map_a)
        }
        (Value::Array(mut arr_a), Value::Array(arr_b)) => {
            arr_a.extend(arr_b);
            Value::Array(arr_a)
        }
        (_, b_value) => b_value,
    }
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

        merge_json_files(&[file1.path(), file2.path()], output.path()).unwrap();
        let content = fs::read_to_string(output.path()).unwrap();
        let parsed: Value = serde_json::from_str(&content).unwrap();

        assert_eq!(parsed["a"], 1);
        assert_eq!(parsed["b"], 2);
        assert_eq!(parsed["c"], 3);
        assert_eq!(parsed["d"], 4);
    }

    #[test]
    fn test_conflict_resolution() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        let output = NamedTempFile::new().unwrap();

        fs::write(&file1, r#"{"a": 1, "b": {"x": 10}}"#).unwrap();
        fs::write(&file2, r#"{"a": 2, "b": {"y": 20}}"#).unwrap();

        merge_json_files(&[file1.path(), file2.path()], output.path()).unwrap();
        let content = fs::read_to_string(output.path()).unwrap();
        let parsed: Value = serde_json::from_str(&content).unwrap();

        assert_eq!(parsed["a"], 2);
        assert_eq!(parsed["b"]["x"], 10);
        assert_eq!(parsed["b"]["y"], 20);
    }
}use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub fn merge_json_files(file_paths: &[&str], output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged_array = Vec::new();
    let mut seen_ids = HashMap::new();

    for file_path in file_paths {
        let content = fs::read_to_string(file_path)?;
        let json_value: Value = serde_json::from_str(&content)?;

        if let Value::Array(arr) = json_value {
            for item in arr {
                if let Some(obj) = item.as_object() {
                    if let Some(id_value) = obj.get("id") {
                        if let Some(id_str) = id_value.as_str() {
                            if seen_ids.contains_key(id_str) {
                                eprintln!("Duplicate ID '{}' found in {}, skipping.", id_str, file_path);
                                continue;
                            }
                            seen_ids.insert(id_str.to_string(), ());
                        }
                    }
                }
                merged_array.push(item);
            }
        } else {
            return Err("Each JSON file must contain a JSON array at its root.".into());
        }
    }

    let output_json = json!(merged_array);
    fs::write(output_path, output_json.to_string())?;
    println!("Successfully merged {} files into {}", file_paths.len(), output_path);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_files() {
        let file1_content = r#"[{"id": "a", "value": 1}, {"id": "b", "value": 2}]"#;
        let file2_content = r#"[{"id": "c", "value": 3}]"#;

        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();

        fs::write(file1.path(), file1_content).unwrap();
        fs::write(file2.path(), file2_content).unwrap();

        let paths = vec![file1.path().to_str().unwrap(), file2.path().to_str().unwrap()];
        let result = merge_json_files(&paths, output_file.path().to_str().unwrap());

        assert!(result.is_ok());

        let output_content = fs::read_to_string(output_file.path()).unwrap();
        let parsed: Value = serde_json::from_str(&output_content).unwrap();
        assert_eq!(parsed.as_array().unwrap().len(), 3);
    }
}