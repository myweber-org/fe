use serde_json::{Value, Map};
use std::collections::HashSet;

pub fn merge_json(base: &mut Value, extension: &Value, overwrite_arrays: bool) {
    match (base, extension) {
        (Value::Object(base_map), Value::Object(extension_map)) => {
            for (key, ext_value) in extension_map {
                if let Some(base_value) = base_map.get_mut(key) {
                    merge_json(base_value, ext_value, overwrite_arrays);
                } else {
                    base_map.insert(key.clone(), ext_value.clone());
                }
            }
        }
        (Value::Array(base_arr), Value::Array(ext_arr)) => {
            if overwrite_arrays {
                *base_arr = ext_arr.clone();
            } else {
                let mut seen = HashSet::new();
                for item in base_arr.iter() {
                    if let Value::Object(map) = item {
                        if let Some(Value::String(id)) = map.get("id") {
                            seen.insert(id.clone());
                        }
                    }
                }
                
                for item in ext_arr {
                    if let Value::Object(map) = item {
                        if let Some(Value::String(id)) = map.get("id") {
                            if !seen.contains(id) {
                                base_arr.push(item.clone());
                            }
                        } else {
                            base_arr.push(item.clone());
                        }
                    } else {
                        base_arr.push(item.clone());
                    }
                }
            }
        }
        (base, extension) => {
            *base = extension.clone();
        }
    }
}

pub fn merge_json_with_strategy(
    base: &str,
    extension: &str,
    overwrite_arrays: bool
) -> Result<String, Box<dyn std::error::Error>> {
    let mut base_value: Value = serde_json::from_str(base)?;
    let extension_value: Value = serde_json::from_str(extension)?;
    
    merge_json(&mut base_value, &extension_value, overwrite_arrays);
    
    Ok(serde_json::to_string_pretty(&base_value)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_merge() {
        let base = r#"{"name": "Alice", "age": 30}"#;
        let extension = r#"{"city": "New York", "age": 31}"#;
        
        let result = merge_json_with_strategy(base, extension, false).unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        
        assert_eq!(parsed["name"], "Alice");
        assert_eq!(parsed["age"], 31);
        assert_eq!(parsed["city"], "New York");
    }
    
    #[test]
    fn test_nested_merge() {
        let base = r#"{"user": {"name": "Alice", "settings": {"theme": "dark"}}}"#;
        let extension = r#"{"user": {"settings": {"language": "en", "theme": "light"}}}"#;
        
        let result = merge_json_with_strategy(base, extension, false).unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        
        assert_eq!(parsed["user"]["name"], "Alice");
        assert_eq!(parsed["user"]["settings"]["theme"], "light");
        assert_eq!(parsed["user"]["settings"]["language"], "en");
    }
}
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

use serde_json::{Map, Value};

pub fn merge_json_files(file_paths: &[&str]) -> Result<Value, Box<dyn std::error::Error>> {
    let mut merged_map = Map::new();

    for path_str in file_paths {
        let path = Path::new(path_str);
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        let mut contents = String::new();
        reader.read_to_string(&mut contents)?;

        let json_value: Value = serde_json::from_str(&contents)?;

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

pub fn merge_json_with_overwrite(
    file_paths: &[&str],
    conflict_resolver: fn(&str, &Value, &Value) -> Value,
) -> Result<Value, Box<dyn std::error::Error>> {
    let mut merged_map: HashMap<String, Value> = HashMap::new();

    for path_str in file_paths {
        let path = Path::new(path_str);
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        let mut contents = String::new();
        reader.read_to_string(&mut contents)?;

        let json_value: Value = serde_json::from_str(&contents)?;

        if let Value::Object(map) = json_value {
            for (key, value) in map {
                if let Some(existing) = merged_map.get(&key) {
                    let resolved = conflict_resolver(&key, existing, &value);
                    merged_map.insert(key, resolved);
                } else {
                    merged_map.insert(key, value);
                }
            }
        } else {
            return Err("Each JSON file must contain a JSON object".into());
        }
    }

    let final_map: Map<String, Value> = merged_map.into_iter().collect();
    Ok(Value::Object(final_map))
}