
use serde_json::{Value, Map};
use std::fs;
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P]) -> Result<Value, Box<dyn std::error::Error>> {
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
    
    Ok(Value::Object(merged))
}

fn merge_value(map: &mut Map<String, Value>, key: String, new_value: Value) {
    match map.get_mut(&key) {
        Some(existing_value) => {
            if let (Value::Object(existing_obj), Value::Object(new_obj)) = (existing_value, &new_value) {
                let mut existing_obj = existing_obj.clone();
                for (k, v) in new_obj {
                    merge_value(&mut existing_obj, k.clone(), v.clone());
                }
                map.insert(key, Value::Object(existing_obj));
            } else if existing_value != &new_value {
                let conflict_array = match existing_value {
                    Value::Array(arr) => {
                        let mut arr = arr.clone();
                        arr.push(new_value.clone());
                        arr
                    }
                    _ => vec![existing_value.clone(), new_value.clone()],
                };
                map.insert(key, Value::Array(conflict_array));
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
    use serde_json::json;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_files() -> Result<(), Box<dyn std::error::Error>> {
        let file1 = NamedTempFile::new()?;
        let file2 = NamedTempFile::new()?;
        
        fs::write(&file1, r#"{"name": "Alice", "age": 30}"#)?;
        fs::write(&file2, r#"{"name": "Bob", "city": "New York"}"#)?;
        
        let result = merge_json_files(&[file1.path(), file2.path()])?;
        
        assert!(result["name"].is_array());
        assert_eq!(result["age"], 30);
        assert_eq!(result["city"], "New York");
        
        Ok(())
    }
}