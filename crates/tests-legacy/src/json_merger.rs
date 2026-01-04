
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub fn merge_json_files(file_paths: &[&str]) -> Result<Value, Box<dyn std::error::Error>> {
    let mut merged_map = Map::new();

    for path_str in file_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            continue;
        }

        let content = fs::read_to_string(path)?;
        let json_value: Value = serde_json::from_str(&content)?;

        if let Value::Object(map) = json_value {
            for (key, value) in map {
                merged_map.insert(key, value);
            }
        }
    }

    Ok(Value::Object(merged_map))
}

pub fn merge_json_with_strategy(
    file_paths: &[&str],
    conflict_strategy: ConflictStrategy,
) -> Result<Value, Box<dyn std::error::Error>> {
    let mut accumulator: HashMap<String, Value> = HashMap::new();

    for path_str in file_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            continue;
        }

        let content = fs::read_to_string(path)?;
        let json_value: Value = serde_json::from_str(&content)?;

        if let Value::Object(map) = json_value {
            for (key, value) in map {
                match conflict_strategy {
                    ConflictStrategy::Overwrite => {
                        accumulator.insert(key, value);
                    }
                    ConflictStrategy::Skip => {
                        accumulator.entry(key).or_insert(value);
                    }
                    ConflictStrategy::MergeObjects => {
                        if let Some(existing) = accumulator.get_mut(&key) {
                            if let (Value::Object(existing_map), Value::Object(new_map)) =
                                (existing, &value)
                            {
                                let mut merged = existing_map.clone();
                                for (k, v) in new_map {
                                    merged.insert(k.clone(), v.clone());
                                }
                                *existing = Value::Object(merged);
                            } else {
                                accumulator.insert(key, value);
                            }
                        } else {
                            accumulator.insert(key, value);
                        }
                    }
                }
            }
        }
    }

    let final_map: Map<String, Value> = accumulator.into_iter().collect();
    Ok(Value::Object(final_map))
}

#[derive(Debug, Clone, Copy)]
pub enum ConflictStrategy {
    Overwrite,
    Skip,
    MergeObjects,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_temp_json(content: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "{}", content).unwrap();
        file
    }

    #[test]
    fn test_merge_basic() {
        let file1 = create_temp_json(r#"{"a": 1, "b": 2}"#);
        let file2 = create_temp_json(r#"{"c": 3, "d": 4}"#);

        let result = merge_json_files(&[
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ])
        .unwrap();

        let expected = json!({
            "a": 1,
            "b": 2,
            "c": 3,
            "d": 4
        });

        assert_eq!(result, expected);
    }

    #[test]
    fn test_conflict_overwrite() {
        let file1 = create_temp_json(r#"{"a": 1, "b": 2}"#);
        let file2 = create_temp_json(r#"{"b": 99, "c": 3}"#);

        let result = merge_json_with_strategy(
            &[
                file1.path().to_str().unwrap(),
                file2.path().to_str().unwrap(),
            ],
            ConflictStrategy::Overwrite,
        )
        .unwrap();

        let expected = json!({
            "a": 1,
            "b": 99,
            "c": 3
        });

        assert_eq!(result, expected);
    }

    #[test]
    fn test_conflict_skip() {
        let file1 = create_temp_json(r#"{"a": 1, "b": 2}"#);
        let file2 = create_temp_json(r#"{"b": 99, "c": 3}"#);

        let result = merge_json_with_strategy(
            &[
                file1.path().to_str().unwrap(),
                file2.path().to_str().unwrap(),
            ],
            ConflictStrategy::Skip,
        )
        .unwrap();

        let expected = json!({
            "a": 1,
            "b": 2,
            "c": 3
        });

        assert_eq!(result, expected);
    }
}
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, Read};
use std::path::Path;

pub fn merge_json_files(file_paths: &[&str], output_path: &str) -> Result<(), String> {
    let mut merged_array = Vec::new();
    let mut seen_keys = HashMap::new();

    for file_path in file_paths {
        let path = Path::new(file_path);
        if !path.exists() {
            return Err(format!("File not found: {}", file_path));
        }

        let file = File::open(path).map_err(|e| e.to_string())?;
        let mut reader = BufReader::new(file);
        let mut contents = String::new();
        reader.read_to_string(&mut contents).map_err(|e| e.to_string())?;

        let json_value: Value = serde_json::from_str(&contents).map_err(|e| e.to_string())?;

        match json_value {
            Value::Array(arr) => {
                for item in arr {
                    if let Some(obj) = item.as_object() {
                        if let Some(id) = obj.get("id").and_then(|v| v.as_str()) {
                            if !seen_keys.contains_key(id) {
                                seen_keys.insert(id.to_string(), true);
                                merged_array.push(item);
                            }
                        } else {
                            merged_array.push(item);
                        }
                    } else {
                        merged_array.push(item);
                    }
                }
            }
            Value::Object(obj) => {
                if let Some(id) = obj.get("id").and_then(|v| v.as_str()) {
                    if !seen_keys.contains_key(id) {
                        seen_keys.insert(id.to_string(), true);
                        merged_array.push(json!(obj));
                    }
                } else {
                    merged_array.push(json!(obj));
                }
            }
            _ => merged_array.push(json_value),
        }
    }

    let output_json = json!(merged_array);
    let output_str = serde_json::to_string_pretty(&output_json).map_err(|e| e.to_string())?;
    fs::write(output_path, output_str).map_err(|e| e.to_string())?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_files() {
        let file1_content = r#"[{"id": "1", "name": "Alice"}, {"id": "2", "name": "Bob"}]"#;
        let file2_content = r#"[{"id": "2", "name": "Robert"}, {"id": "3", "name": "Charlie"}]"#;

        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();

        fs::write(file1.path(), file1_content).unwrap();
        fs::write(file2.path(), file2_content).unwrap();

        let result = merge_json_files(
            &[file1.path().to_str().unwrap(), file2.path().to_str().unwrap()],
            output_file.path().to_str().unwrap(),
        );

        assert!(result.is_ok());

        let output_content = fs::read_to_string(output_file.path()).unwrap();
        let parsed: Value = serde_json::from_str(&output_content).unwrap();

        assert_eq!(parsed.as_array().unwrap().len(), 3);
        assert!(parsed.as_array().unwrap().iter().any(|v| v["id"] == "1"));
        assert!(parsed.as_array().unwrap().iter().any(|v| v["id"] == "2"));
        assert!(parsed.as_array().unwrap().iter().any(|v| v["id"] == "3"));
    }
}