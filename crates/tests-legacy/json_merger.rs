use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, BufReader, Write};
use std::path::Path;

use serde_json::{Map, Value};

fn merge_json_objects(base: &mut Map<String, Value>, new: Map<String, Value>) {
    for (key, value) in new {
        if let Some(existing) = base.get_mut(&key) {
            if existing.is_object() && value.is_object() {
                if let (Some(existing_obj), Some(new_obj)) = (existing.as_object_mut(), value.as_object()) {
                    let mut new_map = Map::new();
                    for (k, v) in new_obj {
                        new_map.insert(k.clone(), v.clone());
                    }
                    merge_json_objects(existing_obj, new_map);
                }
            } else {
                base.insert(key, value);
            }
        } else {
            base.insert(key, value);
        }
    }
}

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P], output_path: P) -> io::Result<()> {
    let mut merged: Map<String, Value> = Map::new();

    for path in paths {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let json: Map<String, Value> = serde_json::from_reader(reader)?;
        merge_json_objects(&mut merged, json);
    }

    let output_file = File::create(output_path)?;
    serde_json::to_writer_pretty(output_file, &merged)?;
    Ok(())
}

pub fn merge_json_from_directory<P: AsRef<Path>>(dir_path: P, output_path: P) -> io::Result<()> {
    let mut json_paths = Vec::new();
    for entry in fs::read_dir(dir_path)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) == Some("json") {
            json_paths.push(path);
        }
    }

    if json_paths.is_empty() {
        return Err(io::Error::new(io::ErrorKind::NotFound, "No JSON files found in directory"));
    }

    merge_json_files(&json_paths, output_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_files() {
        let json1 = r#"{"name": "Alice", "age": 30, "address": {"city": "Paris"}}"#;
        let json2 = r#"{"name": "Bob", "age": 25, "address": {"country": "France"}}"#;

        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();
        file1.write_all(json1.as_bytes()).unwrap();
        file2.write_all(json2.as_bytes()).unwrap();

        let output_file = NamedTempFile::new().unwrap();
        let paths = [file1.path(), file2.path()];

        merge_json_files(&paths, output_file.path()).unwrap();

        let content = fs::read_to_string(output_file.path()).unwrap();
        let parsed: Value = serde_json::from_str(&content).unwrap();

        assert_eq!(parsed["name"], "Bob");
        assert_eq!(parsed["age"], 25);
        assert_eq!(parsed["address"]["city"], "Paris");
        assert_eq!(parsed["address"]["country"], "France");
    }
}
use serde_json::{Value, Map};
use std::fs;
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P]) -> Result<Value, String> {
    if paths.is_empty() {
        return Err("No input files provided".to_string());
    }

    let mut merged_map = Map::new();

    for path in paths {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read {:?}: {}", path.as_ref(), e))?;
        
        let json_value: Value = serde_json::from_str(&content)
            .map_err(|e| format!("Invalid JSON in {:?}: {}", path.as_ref(), e))?;

        if let Value::Object(obj) = json_value {
            merge_objects(&mut merged_map, obj);
        } else {
            return Err(format!("Top-level element in {:?} must be a JSON object", path.as_ref()));
        }
    }

    Ok(Value::Object(merged_map))
}

fn merge_objects(target: &mut Map<String, Value>, source: Map<String, Value>) {
    for (key, source_value) in source {
        match target.get_mut(&key) {
            Some(target_value) => {
                if let (Value::Object(target_obj), Value::Object(source_obj)) = (target_value, &source_value) {
                    let mut target_obj = target_obj.clone();
                    merge_objects(&mut target_obj, source_obj.clone());
                    target.insert(key, Value::Object(target_obj));
                } else if target_value != &source_value {
                    eprintln!("Conflict detected for key '{}', using source value", key);
                    target.insert(key, source_value);
                }
            }
            None => {
                target.insert(key, source_value);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_basic_objects() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();

        fs::write(&file1, r#"{"a": 1, "b": 2}"#).unwrap();
        fs::write(&file2, r#"{"c": 3, "d": 4}"#).unwrap();

        let result = merge_json_files(&[file1.path(), file2.path()]).unwrap();
        let expected = json!({
            "a": 1,
            "b": 2,
            "c": 3,
            "d": 4
        });

        assert_eq!(result, expected);
    }

    #[test]
    fn test_merge_with_nested_conflict() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();

        fs::write(&file1, r#"{"config": {"timeout": 30}}"#).unwrap();
        fs::write(&file2, r#"{"config": {"retries": 5}}"#).unwrap();

        let result = merge_json_files(&[file1.path(), file2.path()]).unwrap();
        let expected = json!({
            "config": {
                "timeout": 30,
                "retries": 5
            }
        });

        assert_eq!(result, expected);
    }
}