
use serde_json::{Value, Map};
use std::fs;
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P]) -> Result<Value, String> {
    let mut merged = Map::new();

    for path in paths {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read {}: {}", path.as_ref().display(), e))?;
        
        let json: Value = serde_json::from_str(&content)
            .map_err(|e| format!("Invalid JSON in {}: {}", path.as_ref().display(), e))?;
        
        if let Value::Object(obj) = json {
            merge_objects(&mut merged, obj);
        } else {
            return Err("Top-level JSON must be an object".to_string());
        }
    }

    Ok(Value::Object(merged))
}

fn merge_objects(target: &mut Map<String, Value>, source: Map<String, Value>) {
    for (key, source_value) in source {
        match target.get_mut(&key) {
            Some(Value::Object(target_obj)) => {
                if let Value::Object(source_obj) = source_value {
                    merge_objects(target_obj, source_obj);
                } else {
                    target.insert(key, source_value);
                }
            }
            Some(Value::Array(target_arr)) => {
                if let Value::Array(source_arr) = source_value {
                    target_arr.extend(source_arr);
                } else {
                    target.insert(key, source_value);
                }
            }
            Some(_) => {
                target.insert(key, source_value);
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
    fn test_merge_nested_objects() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();

        fs::write(&file1, r#"{"config": {"port": 8080}}"#).unwrap();
        fs::write(&file2, r#"{"config": {"host": "localhost"}}"#).unwrap();

        let result = merge_json_files(&[file1.path(), file2.path()]).unwrap();
        let expected = json!({
            "config": {
                "port": 8080,
                "host": "localhost"
            }
        });

        assert_eq!(result, expected);
    }
}
use serde_json::{Value, Map};
use std::fs;
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P], output_path: P) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged = Map::new();
    
    for path in paths {
        let content = fs::read_to_string(path)?;
        let json: Value = serde_json::from_str(&content)?;
        
        if let Value::Object(obj) = json {
            merge_objects(&mut merged, obj);
        }
    }
    
    let output_json = Value::Object(merged);
    let serialized = serde_json::to_string_pretty(&output_json)?;
    fs::write(output_path, serialized)?;
    
    Ok(())
}

fn merge_objects(base: &mut Map<String, Value>, new: Map<String, Value>) {
    for (key, value) in new {
        if !base.contains_key(&key) {
            base.insert(key, value);
        } else {
            match (base.get(&key).unwrap(), value) {
                (Value::Object(mut existing_obj), Value::Object(new_obj)) => {
                    merge_objects(&mut existing_obj, new_obj);
                }
                (Value::Array(existing_arr), Value::Array(new_arr)) => {
                    let mut combined = existing_arr.clone();
                    combined.extend(new_arr);
                    base.insert(key, Value::Array(combined));
                }
                (_, new_value) => {
                    base.insert(key, new_value);
                }
            }
        }
    }
}