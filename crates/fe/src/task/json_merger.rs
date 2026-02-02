
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
            return Err("Each JSON file must contain a JSON object at the root".into());
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
        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();

        writeln!(file1, r#"{"name": "Alice", "age": 30}"#).unwrap();
        writeln!(file2, r#"{"city": "Berlin", "age": 31}"#).unwrap();

        let result = merge_json_files(&[file1.path().to_str().unwrap(), file2.path().to_str().unwrap()]);
        assert!(result.is_ok());

        let merged = result.unwrap();
        assert_eq!(merged["name"], "Alice");
        assert_eq!(merged["age"], 31);
        assert_eq!(merged["city"], "Berlin");
    }

    #[test]
    fn test_file_not_found() {
        let result = merge_json_files(&["nonexistent.json"]);
        assert!(result.is_err());
    }
}
use serde_json::{Value, Map};
use std::fs;
use std::path::Path;
use std::collections::HashSet;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P]) -> Result<Value, String> {
    if paths.is_empty() {
        return Err("No input files provided".to_string());
    }

    let mut merged = Map::new();
    let mut conflict_log = Vec::new();

    for path in paths {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read {}: {}", path.as_ref().display(), e))?;
        
        let json: Value = serde_json::from_str(&content)
            .map_err(|e| format!("Invalid JSON in {}: {}", path.as_ref().display(), e))?;

        merge_value(&mut merged, json, &mut conflict_log, "");
    }

    if !conflict_log.is_empty() {
        eprintln!("Conflicts detected during merge:");
        for conflict in &conflict_log {
            eprintln!("  - {}", conflict);
        }
    }

    Ok(Value::Object(merged))
}

fn merge_value(current: &mut Map<String, Value>, new: Value, conflicts: &mut Vec<String>, path: &str) {
    match new {
        Value::Object(new_map) => {
            for (key, new_val) in new_map {
                let full_path = if path.is_empty() {
                    key.clone()
                } else {
                    format!("{}.{}", path, key)
                };

                match current.get_mut(&key) {
                    Some(Value::Object(existing_map)) => {
                        if let Value::Object(new_child) = new_val {
                            merge_value(existing_map, Value::Object(new_child), conflicts, &full_path);
                        } else {
                            conflicts.push(format!("Type mismatch at '{}'", full_path));
                            current.insert(key, new_val);
                        }
                    }
                    Some(existing) => {
                        if existing != &new_val {
                            conflicts.push(format!("Value conflict at '{}'", full_path));
                        }
                        current.insert(key, new_val);
                    }
                    None => {
                        current.insert(key, new_val);
                    }
                }
            }
        }
        _ => {
            conflicts.push(format!("Root must be object, found {} at '{}'", new, path));
        }
    }
}

pub fn deduplicate_array(array: &mut Value) {
    if let Value::Array(arr) = array {
        let mut seen = HashSet::new();
        arr.retain(|item| seen.insert(item.to_string()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_basic_merge() {
        let file1 = json!({
            "name": "project",
            "version": "1.0.0",
            "dependencies": {
                "serde": "1.0"
            }
        });

        let file2 = json!({
            "version": "1.0.1",
            "dependencies": {
                "tokio": "1.0"
            },
            "author": "Alice"
        });

        let mut map1 = serde_json::to_value(file1).unwrap().as_object().unwrap().clone();
        let val2 = serde_json::to_value(file2).unwrap();
        
        let mut conflicts = Vec::new();
        merge_value(&mut map1, val2, &mut conflicts, "");

        assert_eq!(map1.get("version").unwrap(), "1.0.1");
        assert!(map1.contains_key("author"));
        assert!(conflicts.is_empty());
    }
}