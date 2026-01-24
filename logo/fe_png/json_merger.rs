use std::collections::HashMap;
use serde_json::{Value, Map};

pub fn merge_json(base: &mut Value, update: &Value) {
    match (base, update) {
        (Value::Object(base_map), Value::Object(update_map)) => {
            for (key, update_value) in update_map {
                if let Some(base_value) = base_map.get_mut(key) {
                    merge_json(base_value, update_value);
                } else {
                    base_map.insert(key.clone(), update_value.clone());
                }
            }
        }
        (base, update) => {
            *base = update.clone();
        }
    }
}

pub fn merge_multiple_json(json_objects: Vec<Value>) -> Option<Value> {
    if json_objects.is_empty() {
        return None;
    }

    let mut result = json_objects[0].clone();
    for json in json_objects.iter().skip(1) {
        merge_json(&mut result, json);
    }

    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_basic_merge() {
        let mut base = json!({"a": 1, "b": {"c": 2}});
        let update = json!({"b": {"d": 3}, "e": 4});
        
        merge_json(&mut base, &update);
        
        assert_eq!(base["a"], 1);
        assert_eq!(base["b"]["c"], 2);
        assert_eq!(base["b"]["d"], 3);
        assert_eq!(base["e"], 4);
    }

    #[test]
    fn test_overwrite_primitive() {
        let mut base = json!({"a": 1});
        let update = json!({"a": 2});
        
        merge_json(&mut base, &update);
        
        assert_eq!(base["a"], 2);
    }

    #[test]
    fn test_merge_multiple() {
        let objects = vec![
            json!({"a": 1}),
            json!({"b": 2}),
            json!({"a": 3, "c": 4})
        ];
        
        let result = merge_multiple_json(objects).unwrap();
        
        assert_eq!(result["a"], 3);
        assert_eq!(result["b"], 2);
        assert_eq!(result["c"], 4);
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
        writeln!(file2, r#"{"city": "Berlin", "active": true}"#).unwrap();

        let paths = [
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ];

        let result = merge_json_files(&paths).unwrap();
        let obj = result.as_object().unwrap();

        assert_eq!(obj.get("name").unwrap(), "Alice");
        assert_eq!(obj.get("age").unwrap(), 30);
        assert_eq!(obj.get("city").unwrap(), "Berlin");
        assert_eq!(obj.get("active").unwrap(), true);
    }
}