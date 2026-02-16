
use serde_json::{Value, Map};
use std::fs;
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P]) -> Result<Value, String> {
    let mut merged = Map::new();

    for path in paths {
        let content = fs::read_to_string(path).map_err(|e| format!("Failed to read {}: {}", path.as_ref().display(), e))?;
        let json: Value = serde_json::from_str(&content).map_err(|e| format!("Invalid JSON in {}: {}", path.as_ref().display(), e))?;

        if let Value::Object(obj) = json {
            for (key, value) in obj {
                if merged.contains_key(&key) {
                    let existing = merged.get(&key).unwrap();
                    if existing != &value {
                        return Err(format!("Conflict detected for key '{}' between files", key));
                    }
                } else {
                    merged.insert(key, value);
                }
            }
        } else {
            return Err("Top-level JSON must be an object".to_string());
        }
    }

    Ok(Value::Object(merged))
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

        writeln!(file1, r#"{"a": 1, "b": "test"}"#).unwrap();
        writeln!(file2, r#"{"c": true, "d": [1,2,3]}"#).unwrap();

        let result = merge_json_files(&[file1.path(), file2.path()]).unwrap();
        let obj = result.as_object().unwrap();

        assert_eq!(obj.len(), 4);
        assert_eq!(obj.get("a").unwrap().as_i64(), Some(1));
        assert_eq!(obj.get("b").unwrap().as_str(), Some("test"));
        assert_eq!(obj.get("c").unwrap().as_bool(), Some(true));
        assert!(obj.get("d").unwrap().is_array());
    }

    #[test]
    fn test_merge_conflict() {
        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();

        writeln!(file1, r#"{"key": "value1"}"#).unwrap();
        writeln!(file2, r#"{"key": "value2"}"#).unwrap();

        let result = merge_json_files(&[file1.path(), file2.path()]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Conflict"));
    }
}use serde_json::{Map, Value};
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

        let paths = [
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ];

        let result = merge_json_files(&paths).unwrap();
        let obj = result.as_object().unwrap();

        assert_eq!(obj.get("name").unwrap(), "Alice");
        assert_eq!(obj.get("age").unwrap(), 30);
        assert_eq!(obj.get("city").unwrap(), "London");
        assert_eq!(obj.get("active").unwrap(), true);
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

    Ok(Value::Array(merged_array))
}

pub fn write_merged_json<P: AsRef<Path>>(output_path: P, value: &Value) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::create(output_path)?;
    serde_json::to_writer_pretty(file, value)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_arrays() {
        let file1_content = json!([{"id": 1, "name": "Alice"}, {"id": 2, "name": "Bob"}]);
        let file2_content = json!([{"id": 3, "name": "Charlie"}]);

        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();

        write!(file1, "{}", file1_content).unwrap();
        write!(file2, "{}", file2_content).unwrap();

        let result = merge_json_files(&[file1.path(), file2.path()], Some("id")).unwrap();
        assert_eq!(result.as_array().unwrap().len(), 3);
    }

    #[test]
    fn test_deduplicate_by_key() {
        let file1_content = json!([{"id": 1, "name": "Alice"}, {"id": 2, "name": "Bob"}]);
        let file2_content = json!([{"id": 1, "name": "Alice Duplicate"}, {"id": 3, "name": "Charlie"}]);

        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();

        write!(file1, "{}", file1_content).unwrap();
        write!(file2, "{}", file2_content).unwrap();

        let result = merge_json_files(&[file1.path(), file2.path()], Some("id")).unwrap();
        let array = result.as_array().unwrap();
        assert_eq!(array.len(), 3);

        let ids: Vec<i64> = array
            .iter()
            .filter_map(|v| v.get("id").and_then(|id| id.as_i64()))
            .collect();
        assert_eq!(ids, vec![1, 2, 3]);
    }
}
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P], output_path: P) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged_map = Map::new();
    let mut key_counter: HashMap<String, usize> = HashMap::new();

    for path in paths {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let json_data: Value = serde_json::from_reader(reader)?;

        if let Value::Object(obj) = json_data {
            for (key, value) in obj {
                let mut final_key = key.clone();
                while merged_map.contains_key(&final_key) {
                    let count = key_counter.entry(key.clone()).or_insert(1);
                    *count += 1;
                    final_key = format!("{}_{}", key, count);
                }
                merged_map.insert(final_key, value);
            }
        } else {
            return Err("Each JSON file must contain a JSON object".into());
        }
    }

    let merged_value = Value::Object(merged_map);
    let mut output_file = File::create(output_path)?;
    write!(output_file, "{}", serde_json::to_string_pretty(&merged_value)?)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_files() {
        let json1 = r#"{"name": "Alice", "age": 30}"#;
        let json2 = r#"{"city": "Berlin", "name": "Bob"}"#;

        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();

        write!(file1, "{}", json1).unwrap();
        write!(file2, "{}", json2).unwrap();

        let paths = [file1.path(), file2.path()];
        merge_json_files(&paths, output_file.path()).unwrap();

        let output_content = std::fs::read_to_string(output_file.path()).unwrap();
        let parsed: Value = serde_json::from_str(&output_content).unwrap();

        assert!(parsed.get("name").is_some());
        assert!(parsed.get("name_2").is_some());
        assert!(parsed.get("age").is_some());
        assert!(parsed.get("city").is_some());
    }
}