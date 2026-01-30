
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
}