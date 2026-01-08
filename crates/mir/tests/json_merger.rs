use serde_json::{Map, Value};

pub fn merge_json(a: &mut Value, b: &Value) {
    match (a, b) {
        (Value::Object(a_map), Value::Object(b_map)) => {
            for (key, b_value) in b_map {
                if let Some(a_value) = a_map.get_mut(key) {
                    merge_json(a_value, b_value);
                } else {
                    a_map.insert(key.clone(), b_value.clone());
                }
            }
        }
        (a, b) => *a = b.clone(),
    }
}

pub fn merge_json_with_strategy(a: &mut Value, b: &Value, strategy: MergeStrategy) {
    match (a, b) {
        (Value::Object(a_map), Value::Object(b_map)) => {
            for (key, b_value) in b_map {
                if let Some(a_value) = a_map.get_mut(key) {
                    merge_json_with_strategy(a_value, b_value, strategy.clone());
                } else {
                    a_map.insert(key.clone(), b_value.clone());
                }
            }
        }
        (Value::Array(a_arr), Value::Array(b_arr)) => {
            match strategy {
                MergeStrategy::Replace => *a_arr = b_arr.clone(),
                MergeStrategy::Append => a_arr.extend(b_arr.clone()),
                MergeStrategy::Merge => {
                    for (i, b_item) in b_arr.iter().enumerate() {
                        if i < a_arr.len() {
                            merge_json_with_strategy(&mut a_arr[i], b_item, strategy.clone());
                        } else {
                            a_arr.push(b_item.clone());
                        }
                    }
                }
            }
        }
        (a, b) => *a = b.clone(),
    }
}

#[derive(Clone)]
pub enum MergeStrategy {
    Replace,
    Append,
    Merge,
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
            Some(target_value) => {
                if let (Value::Object(mut target_obj), Value::Object(source_obj)) = (target_value.clone(), source_value) {
                    merge_objects(&mut target_obj, source_obj);
                    target.insert(key, Value::Object(target_obj));
                } else {
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
    fn test_merge_json() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        
        fs::write(&file1, r#"{"a": 1, "nested": {"x": 10}}"#).unwrap();
        fs::write(&file2, r#"{"b": 2, "nested": {"y": 20}}"#).unwrap();
        
        let result = merge_json_files(&[file1.path(), file2.path()]).unwrap();
        
        assert_eq!(result["a"], 1);
        assert_eq!(result["b"], 2);
        assert_eq!(result["nested"]["x"], 10);
        assert_eq!(result["nested"]["y"], 20);
    }
}