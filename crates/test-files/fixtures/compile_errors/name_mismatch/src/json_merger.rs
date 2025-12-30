use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use serde_json::{Map, Value};

pub fn merge_json_files(file_paths: &[&str]) -> Result<Value, Box<dyn std::error::Error>> {
    let mut merged_map = Map::new();

    for path_str in file_paths {
        let path = Path::new(path_str);
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let json_value: Value = serde_json::from_reader(reader)?;

        if let Value::Object(map) = json_value {
            for (key, value) in map {
                merged_map.insert(key, value);
            }
        } else {
            return Err("Each JSON file must contain a JSON object".into());
        }
    }

    Ok(Value::Object(merged_map))
}

pub fn merge_json_with_strategy(
    file_paths: &[&str],
    conflict_strategy: fn(&str, &Value, &Value) -> Value,
) -> Result<Value, Box<dyn std::error::Error>> {
    let mut merged_map: HashMap<String, Value> = HashMap::new();

    for path_str in file_paths {
        let path = Path::new(path_str);
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let json_value: Value = serde_json::from_reader(reader)?;

        if let Value::Object(map) = json_value {
            for (key, value) in map {
                match merged_map.get(&key) {
                    Some(existing_value) => {
                        let resolved_value = conflict_strategy(&key, existing_value, &value);
                        merged_map.insert(key, resolved_value);
                    }
                    None => {
                        merged_map.insert(key, value);
                    }
                }
            }
        } else {
            return Err("Each JSON file must contain a JSON object".into());
        }
    }

    let final_map: Map<String, Value> = merged_map.into_iter().collect();
    Ok(Value::Object(final_map))
}

pub fn default_conflict_strategy(_key: &str, existing: &Value, new: &Value) -> Value {
    if existing.is_array() && new.is_array() {
        let mut combined = Vec::new();
        if let Value::Array(existing_arr) = existing {
            combined.extend_from_slice(existing_arr);
        }
        if let Value::Array(new_arr) = new {
            combined.extend_from_slice(new_arr);
        }
        Value::Array(combined)
    } else {
        new.clone()
    }
}
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
            for (key, value) in obj {
                if merged.contains_key(&key) {
                    let existing = &merged[&key];
                    if existing != &value {
                        return Err(format!("Conflict detected for key '{}'", key));
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
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_merge_valid_files() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        
        fs::write(&file1, r#"{"a": 1, "b": "test"}"#).unwrap();
        fs::write(&file2, r#"{"c": true, "d": [1,2,3]}"#).unwrap();
        
        let result = merge_json_files(&[&file1, &file2]).unwrap();
        let obj = result.as_object().unwrap();
        
        assert_eq!(obj.len(), 4);
        assert_eq!(obj["a"], 1);
        assert_eq!(obj["b"], "test");
        assert_eq!(obj["c"], true);
    }
    
    #[test]
    fn test_merge_with_conflict() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        
        fs::write(&file1, r#"{"key": "value1"}"#).unwrap();
        fs::write(&file2, r#"{"key": "value2"}"#).unwrap();
        
        let result = merge_json_files(&[&file1, &file2]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Conflict"));
    }
}