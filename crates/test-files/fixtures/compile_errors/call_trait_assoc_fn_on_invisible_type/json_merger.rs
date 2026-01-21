
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub fn merge_json_files(file_paths: &[&str]) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let mut merged = HashMap::new();

    for path_str in file_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            return Err(format!("File not found: {}", path_str).into());
        }

        let content = fs::read_to_string(path)?;
        let json_value: serde_json::Value = serde_json::from_str(&content)?;

        if let Some(obj) = json_value.as_object() {
            for (key, value) in obj {
                merged.insert(key.clone(), value.clone());
            }
        } else {
            return Err("Each JSON file must contain an object at the root".into());
        }
    }

    Ok(serde_json::Value::Object(
        merged.into_iter().collect()
    ))
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
        writeln!(file2, r#"{"city": "London", "active": true}"#).unwrap();

        let result = merge_json_files(&[
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ]).unwrap();

        assert_eq!(result["name"], "Alice");
        assert_eq!(result["age"], 30);
        assert_eq!(result["city"], "London");
        assert_eq!(result["active"], true);
    }

    #[test]
    fn test_merge_with_overwrite() {
        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();

        writeln!(file1, r#"{"id": 1, "value": "old"}"#).unwrap();
        writeln!(file2, r#"{"id": 2, "value": "new"}"#).unwrap();

        let result = merge_json_files(&[
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ]).unwrap();

        assert_eq!(result["id"], 2);
        assert_eq!(result["value"], "new");
    }
}use serde_json::{json, Value};
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P], deduplicate_by_key: Option<&str>) -> Result<Value, Box<dyn std::error::Error>> {
    let mut merged_array = Vec::new();
    let mut seen_keys = HashSet::new();

    for path in paths {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        let mut contents = String::new();
        reader.read_to_string(&mut contents)?;

        let json_value: Value = serde_json::from_str(&contents)?;

        match json_value {
            Value::Array(arr) => {
                for item in arr {
                    if let Some(key) = deduplicate_by_key {
                        if let Some(obj) = item.as_object() {
                            if let Some(key_value) = obj.get(key) {
                                let key_str = key_value.to_string();
                                if !seen_keys.contains(&key_str) {
                                    seen_keys.insert(key_str);
                                    merged_array.push(item);
                                }
                                continue;
                            }
                        }
                    }
                    merged_array.push(item);
                }
            }
            _ => merged_array.push(json_value),
        }
    }

    Ok(json!(merged_array))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_basic() {
        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();

        writeln!(file1, r#"[{"id": 1, "name": "Alice"}]"#).unwrap();
        writeln!(file2, r#"[{"id": 2, "name": "Bob"}]"#).unwrap();

        let result = merge_json_files(&[file1.path(), file2.path()], None).unwrap();
        assert_eq!(result.as_array().unwrap().len(), 2);
    }

    #[test]
    fn test_deduplication() {
        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();

        writeln!(file1, r#"[{"id": 1, "name": "Alice"}]"#).unwrap();
        writeln!(file2, r#"[{"id": 1, "name": "Alice"}, {"id": 3, "name": "Charlie"}]"#).unwrap();

        let result = merge_json_files(&[file1.path(), file2.path()], Some("id")).unwrap();
        assert_eq!(result.as_array().unwrap().len(), 2);
    }
}