
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub fn merge_json_files(file_paths: &[&str]) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let mut merged_map = HashMap::new();

    for file_path in file_paths {
        let path = Path::new(file_path);
        if !path.exists() {
            return Err(format!("File not found: {}", file_path).into());
        }

        let content = fs::read_to_string(path)?;
        let json_value: serde_json::Value = serde_json::from_str(&content)?;

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

        writeln!(file1, r#"{"id": 1, "value": "first"}"#).unwrap();
        writeln!(file2, r#"{"id": 2, "extra": "data"}"#).unwrap();

        let result = merge_json_files(&[
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ]).unwrap();

        assert_eq!(result["id"], 2);
        assert_eq!(result["value"], "first");
        assert_eq!(result["extra"], "data");
    }
}use std::collections::HashMap;
use serde_json::{Value, Map};

pub fn merge_json(base: &mut Value, update: &Value) {
    match (base, update) {
        (Value::Object(base_map), Value::Object(update_map)) => {
            for (key, update_value) in update_map {
                if let Some(base_value) = base_map.get_mut(key) {
                    merge_json(base_value, update_value);
                } else {
                    base_map.insert(key.clone(), update_value.clone());
                }
            }
        }
        (base, update) => {
            *base = update.clone();
        }
    }
}

pub fn merge_json_with_strategy(
    base: &mut Value,
    update: &Value,
    array_merge_strategy: ArrayMergeStrategy,
) {
    match (base, update) {
        (Value::Object(base_map), Value::Object(update_map)) => {
            for (key, update_value) in update_map {
                if let Some(base_value) = base_map.get_mut(key) {
                    merge_json_with_strategy(base_value, update_value, array_merge_strategy.clone());
                } else {
                    base_map.insert(key.clone(), update_value.clone());
                }
            }
        }
        (Value::Array(base_arr), Value::Array(update_arr)) => {
            match array_merge_strategy {
                ArrayMergeStrategy::Replace => {
                    *base_arr = update_arr.clone();
                }
                ArrayMergeStrategy::Append => {
                    base_arr.extend(update_arr.clone());
                }
                ArrayMergeStrategy::MergeUnique => {
                    let mut seen = HashMap::new();
                    let mut merged = Vec::new();
                    
                    for item in base_arr.iter().chain(update_arr.iter()) {
                        let key = format!("{:?}", item);
                        if !seen.contains_key(&key) {
                            seen.insert(key, true);
                            merged.push(item.clone());
                        }
                    }
                    
                    *base_arr = merged;
                }
            }
        }
        (base, update) => {
            *base = update.clone();
        }
    }
}

#[derive(Clone)]
pub enum ArrayMergeStrategy {
    Replace,
    Append,
    MergeUnique,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_basic_merge() {
        let mut base = json!({
            "name": "Alice",
            "age": 30,
            "address": {
                "city": "New York",
                "zip": "10001"
            }
        });

        let update = json!({
            "age": 31,
            "address": {
                "zip": "10002",
                "country": "USA"
            },
            "hobbies": ["reading", "coding"]
        });

        merge_json(&mut base, &update);

        assert_eq!(base["age"], 31);
        assert_eq!(base["address"]["zip"], "10002");
        assert_eq!(base["address"]["country"], "USA");
        assert_eq!(base["hobbies"][0], "reading");
    }

    #[test]
    fn test_array_merge_strategies() {
        let mut base = json!({
            "tags": ["rust", "json"]
        });

        let update = json!({
            "tags": ["merge", "utility"]
        });

        merge_json_with_strategy(
            &mut base,
            &update,
            ArrayMergeStrategy::Append,
        );

        assert_eq!(base["tags"].as_array().unwrap().len(), 4);
    }
}