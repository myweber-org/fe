
use serde_json::{Value, Map};
use std::fs;
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P]) -> Result<Value, Box<dyn std::error::Error>> {
    let mut result = Map::new();

    for path in paths {
        let content = fs::read_to_string(path)?;
        let json: Value = serde_json::from_str(&content)?;

        if let Value::Object(obj) = json {
            merge_objects(&mut result, obj);
        }
    }

    Ok(Value::Object(result))
}

fn merge_objects(target: &mut Map<String, Value>, source: Map<String, Value>) {
    for (key, source_value) in source {
        match target.get_mut(&key) {
            Some(target_value) => {
                if let (Value::Object(mut target_obj), Value::Object(source_obj)) = (target_value.clone(), source_value.clone()) {
                    merge_objects(&mut target_obj, source_obj);
                    target.insert(key, Value::Object(target_obj));
                } else if target_value != &source_value {
                    target.insert(format!("{}_conflict", key), source_value);
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
    fn test_merge_json() -> Result<(), Box<dyn std::error::Error>> {
        let file1 = NamedTempFile::new()?;
        let file2 = NamedTempFile::new()?;

        fs::write(&file1, r#"{"a": 1, "b": {"x": 10}}"#)?;
        fs::write(&file2, r#"{"b": {"y": 20}, "c": 3}"#)?;

        let result = merge_json_files(&[file1.path(), file2.path()])?;
        let expected = json!({
            "a": 1,
            "b": {"x": 10, "y": 20},
            "c": 3
        });

        assert_eq!(result, expected);
        Ok(())
    }
}use serde_json::{Map, Value};
use std::fs;
use std::path::Path;

pub fn merge_json_files(file_paths: &[&str]) -> Result<Value, Box<dyn std::error::Error>> {
    let mut merged_map = Map::new();

    for file_path in file_paths {
        let path = Path::new(file_path);
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
    use serde_json::json;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_files() {
        let file1_content = json!({"name": "Alice", "age": 30});
        let file2_content = json!({"city": "Berlin", "active": true});

        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();

        fs::write(file1.path(), file1_content.to_string()).unwrap();
        fs::write(file2.path(), file2_content.to_string()).unwrap();

        let paths = vec![
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ];

        let result = merge_json_files(&paths).unwrap();
        let expected = json!({
            "name": "Alice",
            "age": 30,
            "city": "Berlin",
            "active": true
        });

        assert_eq!(result, expected);
    }

    #[test]
    fn test_merge_with_missing_file() {
        let file_content = json!({"data": "test"});
        let file = NamedTempFile::new().unwrap();
        fs::write(file.path(), file_content.to_string()).unwrap();

        let paths = vec![
            file.path().to_str().unwrap(),
            "non_existent_file.json",
        ];

        let result = merge_json_files(&paths).unwrap();
        assert_eq!(result, json!({"data": "test"}));
    }
}