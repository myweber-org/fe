
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
    match map.get(&key) {
        Some(Value::Object(existing_obj)) => {
            if let Value::Object(new_obj) = new_value {
                let mut merged_obj = existing_obj.clone();
                for (nested_key, nested_value) in new_obj {
                    merge_value(&mut merged_obj, nested_key, nested_value);
                }
                map.insert(key, Value::Object(merged_obj));
            } else {
                map.insert(key, new_value);
            }
        }
        Some(Value::Array(existing_arr)) => {
            if let Value::Array(new_arr) = new_value {
                let mut merged_arr = existing_arr.clone();
                merged_arr.extend(new_arr);
                map.insert(key, Value::Array(merged_arr));
            } else {
                map.insert(key, new_value);
            }
        }
        _ => {
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
    fn test_merge_json() -> Result<(), Box<dyn std::error::Error>> {
        let file1 = NamedTempFile::new()?;
        let file2 = NamedTempFile::new()?;

        fs::write(&file1, r#"{"a": 1, "b": {"x": 10}}"#)?;
        fs::write(&file2, r#"{"b": {"y": 20}, "c": [1,2]}"#)?;

        let result = merge_json_files(&[file1.path(), file2.path()])?;
        
        assert_eq!(result["a"], 1);
        assert_eq!(result["b"]["x"], 10);
        assert_eq!(result["b"]["y"], 20);
        assert_eq!(result["c"], json!([1,2]));

        Ok(())
    }
}