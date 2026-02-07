use serde_json::{Map, Value};
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

        if let Value::Object(map) = json_value {
            for (key, value) in map {
                merged_map.insert(key, value);
            }
        }
    }

    Ok(Value::Object(merged_map))
}

pub fn merge_json_with_strategy(
    file_paths: &[&str],
    conflict_strategy: fn(&str, &Value, &Value) -> Value,
) -> Result<Value, Box<dyn std::error::Error>> {
    let mut accumulator: HashMap<String, Value> = HashMap::new();

    for path_str in file_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            continue;
        }

        let content = fs::read_to_string(path)?;
        let json_value: Value = serde_json::from_str(&content)?;

        if let Value::Object(map) = json_value {
            for (key, value) in map {
                match accumulator.get(&key) {
                    Some(existing_value) => {
                        let resolved_value = conflict_strategy(&key, existing_value, &value);
                        accumulator.insert(key, resolved_value);
                    }
                    None => {
                        accumulator.insert(key, value);
                    }
                }
            }
        }
    }

    let final_map: Map<String, Value> = accumulator.into_iter().collect();
    Ok(Value::Object(final_map))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_temp_json(content: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "{}", content).unwrap();
        file
    }

    #[test]
    fn test_basic_merge() {
        let file1 = create_temp_json(r#"{"a": 1, "b": 2}"#);
        let file2 = create_temp_json(r#"{"c": 3, "d": 4}"#);

        let result = merge_json_files(&[
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ])
        .unwrap();

        let expected = json!({
            "a": 1,
            "b": 2,
            "c": 3,
            "d": 4
        });

        assert_eq!(result, expected);
    }

    #[test]
    fn test_conflict_resolution() {
        let file1 = create_temp_json(r#"{"key": "first"}"#);
        let file2 = create_temp_json(r#"{"key": "second"}"#);

        let strategy = |_key: &str, _old: &Value, new: &Value| new.clone();

        let result = merge_json_with_strategy(
            &[
                file1.path().to_str().unwrap(),
                file2.path().to_str().unwrap(),
            ],
            strategy,
        )
        .unwrap();

        assert_eq!(result["key"], "second");
    }
}use std::collections::HashMap;
use serde_json::{Value, Map};

pub fn deep_merge(target: &mut Value, source: &Value) {
    match (target, source) {
        (Value::Object(t), Value::Object(s)) => {
            let t_map = t.as_object_mut().unwrap();
            let s_map = s.as_object().unwrap();
            
            for (key, value) in s_map {
                if t_map.contains_key(key) {
                    deep_merge(t_map.get_mut(key).unwrap(), value);
                } else {
                    t_map.insert(key.clone(), value.clone());
                }
            }
        }
        (target, source) => {
            *target = source.clone();
        }
    }
}

pub fn merge_multiple(json_objects: Vec<Value>) -> Value {
    let mut result = Value::Object(Map::new());
    
    for obj in json_objects {
        deep_merge(&mut result, &obj);
    }
    
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_basic_merge() {
        let obj1 = json!({"a": 1, "b": {"c": 2}});
        let obj2 = json!({"b": {"d": 3}, "e": 4});
        
        let mut merged = obj1.clone();
        deep_merge(&mut merged, &obj2);
        
        assert_eq!(merged["a"], 1);
        assert_eq!(merged["b"]["c"], 2);
        assert_eq!(merged["b"]["d"], 3);
        assert_eq!(merged["e"], 4);
    }

    #[test]
    fn test_overwrite_primitive() {
        let mut target = json!({"a": 1});
        let source = json!({"a": 2});
        
        deep_merge(&mut target, &source);
        assert_eq!(target["a"], 2);
    }

    #[test]
    fn test_multiple_merge() {
        let objects = vec![
            json!({"a": 1}),
            json!({"b": 2}),
            json!({"a": 3, "c": 4})
        ];
        
        let result = merge_multiple(objects);
        assert_eq!(result["a"], 3);
        assert_eq!(result["b"], 2);
        assert_eq!(result["c"], 4);
    }
}