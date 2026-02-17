
use serde_json::{Value, Map};
use std::fs;
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P], output_path: P) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged = Map::new();
    
    for path in paths {
        let content = fs::read_to_string(path)?;
        let json: Value = serde_json::from_str(&content)?;
        
        if let Value::Object(obj) = json {
            for (key, value) in obj {
                merge_value(&mut merged, key, value);
            }
        }
    }
    
    let output_json = Value::Object(merged);
    let output_str = serde_json::to_string_pretty(&output_json)?;
    fs::write(output_path, output_str)?;
    
    Ok(())
}

fn merge_value(map: &mut Map<String, Value>, key: String, new_value: Value) {
    match map.get(&key) {
        Some(existing_value) => {
            match (existing_value, new_value) {
                (Value::Object(existing_obj), Value::Object(new_obj)) => {
                    let mut merged_obj = existing_obj.clone();
                    for (k, v) in new_obj {
                        merge_value(&mut merged_obj, k, v);
                    }
                    map.insert(key, Value::Object(merged_obj));
                }
                (Value::Array(existing_arr), Value::Array(new_arr)) => {
                    let mut merged_arr = existing_arr.clone();
                    merged_arr.extend(new_arr);
                    map.insert(key, Value::Array(merged_arr));
                }
                _ => {
                    map.insert(key, new_value);
                }
            }
        }
        None => {
            map.insert(key, new_value);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use serde_json::json;

    #[test]
    fn test_merge_json() -> Result<(), Box<dyn std::error::Error>> {
        let file1 = NamedTempFile::new()?;
        let file2 = NamedTempFile::new()?;
        let output = NamedTempFile::new()?;
        
        fs::write(&file1, r#"{"a": 1, "b": {"x": 10}}"#)?;
        fs::write(&file2, r#"{"b": {"y": 20}, "c": 30}"#)?;
        
        merge_json_files(&[file1.path(), file2.path()], output.path())?;
        
        let result_content = fs::read_to_string(output.path())?;
        let result: Value = serde_json::from_str(&result_content)?;
        
        let expected = json!({
            "a": 1,
            "b": {"x": 10, "y": 20},
            "c": 30
        });
        
        assert_eq!(result, expected);
        Ok(())
    }
}use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, Read};
use std::path::Path;

use serde_json::{Map, Value};

pub fn merge_json_files(file_paths: &[&str]) -> Result<Value, Box<dyn std::error::Error>> {
    let mut merged_map = Map::new();

    for file_path in file_paths {
        let path = Path::new(file_path);
        if !path.exists() {
            return Err(format!("File not found: {}", file_path).into());
        }

        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        let mut contents = String::new();
        reader.read_to_string(&mut contents)?;

        let json_value: Value = serde_json::from_str(&contents)?;

        if let Value::Object(obj) = json_value {
            for (key, value) in obj {
                merged_map.insert(key, value);
            }
        } else {
            return Err("JSON file does not contain an object at the root".into());
        }
    }

    Ok(Value::Object(merged_map))
}

pub fn merge_json_directories(dir_paths: &[&str]) -> Result<Value, Box<dyn std::error::Error>> {
    let mut file_paths = Vec::new();

    for dir_path in dir_paths {
        let path = Path::new(dir_path);
        if !path.is_dir() {
            return Err(format!("Directory not found: {}", dir_path).into());
        }

        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let file_path = entry.path();
            if file_path.extension().and_then(|s| s.to_str()) == Some("json") {
                file_paths.push(file_path.to_string_lossy().into_owned());
            }
        }
    }

    let ref_paths: Vec<&str> = file_paths.iter().map(|s| s.as_str()).collect();
    merge_json_files(&ref_paths)
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
        writeln!(file2, r#"{"city": "London", "country": "UK"}"#).unwrap();

        let result = merge_json_files(&[
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ])
        .unwrap();

        let expected: Value = serde_json::from_str(
            r#"{"name": "Alice", "age": 30, "city": "London", "country": "UK"}"#,
        )
        .unwrap();

        assert_eq!(result, expected);
    }

    #[test]
    fn test_merge_json_directories() {
        let dir = tempfile::tempdir().unwrap();
        let file1_path = dir.path().join("data1.json");
        let file2_path = dir.path().join("data2.json");

        fs::write(&file1_path, r#"{"key1": "value1"}"#).unwrap();
        fs::write(&file2_path, r#"{"key2": "value2"}"#).unwrap();

        let result = merge_json_directories(&[dir.path().to_str().unwrap()]).unwrap();

        let expected: Value = serde_json::from_str(r#"{"key1": "value1", "key2": "value2"}"#).unwrap();
        assert_eq!(result, expected);
    }
}use serde_json::{Map, Value};
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

        if let Value::Object(obj) = json_value {
            for (key, value) in obj {
                merged_map.insert(key, value);
            }
        }
    }

    Ok(Value::Object(merged_map))
}

pub fn merge_with_strategy(
    file_paths: &[&str],
    strategy: MergeStrategy,
) -> Result<Value, Box<dyn std::error::Error>> {
    match strategy {
        MergeStrategy::Overwrite => merge_json_files(file_paths),
        MergeStrategy::CombineArrays => merge_with_array_combination(file_paths),
    }
}

fn merge_with_array_combination(file_paths: &[&str]) -> Result<Value, Box<dyn std::error::Error>> {
    let mut combined_map: HashMap<String, Vec<Value>> = HashMap::new();

    for path_str in file_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            continue;
        }

        let content = fs::read_to_string(path)?;
        let json_value: Value = serde_json::from_str(&content)?;

        if let Value::Object(obj) = json_value {
            for (key, value) in obj {
                combined_map
                    .entry(key)
                    .or_insert_with(Vec::new)
                    .push(value);
            }
        }
    }

    let mut result_map = Map::new();
    for (key, values) in combined_map {
        if values.len() == 1 {
            result_map.insert(key, values[0].clone());
        } else {
            let combined_array: Vec<Value> = values
                .into_iter()
                .flat_map(|v| {
                    if let Value::Array(arr) = v {
                        arr
                    } else {
                        vec![v]
                    }
                })
                .collect();
            result_map.insert(key, Value::Array(combined_array));
        }
    }

    Ok(Value::Object(result_map))
}

#[derive(Debug, Clone, Copy)]
pub enum MergeStrategy {
    Overwrite,
    CombineArrays,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_files() {
        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();

        let json1 = json!({"name": "Alice", "age": 30});
        let json2 = json!({"city": "Berlin", "age": 31});

        write!(file1, "{}", json1).unwrap();
        write!(file2, "{}", json2).unwrap();

        let paths = [
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ];
        let result = merge_json_files(&paths).unwrap();

        assert_eq!(result["name"], "Alice");
        assert_eq!(result["city"], "Berlin");
        assert_eq!(result["age"], 31);
    }

    #[test]
    fn test_merge_with_array_strategy() {
        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();

        let json1 = json!({"tags": ["rust", "json"], "id": 1});
        let json2 = json!({"tags": ["merge"], "active": true});

        write!(file1, "{}", json1).unwrap();
        write!(file2, "{}", json2).unwrap();

        let paths = [
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ];
        let result = merge_with_strategy(&paths, MergeStrategy::CombineArrays).unwrap();

        if let Value::Array(tags) = &result["tags"] {
            assert_eq!(tags.len(), 3);
            assert!(tags.contains(&Value::String("rust".to_string())));
            assert!(tags.contains(&Value::String("json".to_string())));
            assert!(tags.contains(&Value::String("merge".to_string())));
        } else {
            panic!("Expected array for 'tags' key");
        }
        assert_eq!(result["id"], 1);
        assert_eq!(result["active"], true);
    }
}