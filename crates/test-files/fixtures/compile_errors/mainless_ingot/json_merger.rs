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
}use serde_json::{Value, Map};
use std::collections::HashSet;

pub fn merge_json(base: &mut Value, extension: &Value, overwrite_arrays: bool) {
    match (base, extension) {
        (Value::Object(base_map), Value::Object(ext_map)) => {
            for (key, ext_value) in ext_map {
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
                                seen.insert(id.clone());
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
        let ext = r#"{"age": 31, "city": "New York"}"#;
        
        let result = merge_json_with_strategy(base, ext, false).unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        
        assert_eq!(parsed["name"], "Alice");
        assert_eq!(parsed["age"], 31);
        assert_eq!(parsed["city"], "New York");
    }
    
    #[test]
    fn test_nested_merge() {
        let base = r#"{"user": {"name": "Bob", "settings": {"theme": "dark"}}}"#;
        let ext = r#"{"user": {"settings": {"language": "en"}}}"#;
        
        let result = merge_json_with_strategy(base, ext, false).unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        
        assert_eq!(parsed["user"]["name"], "Bob");
        assert_eq!(parsed["user"]["settings"]["theme"], "dark");
        assert_eq!(parsed["user"]["settings"]["language"], "en");
    }
}use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, Read};
use std::path::Path;

type JsonValue = serde_json::Value;

pub fn merge_json_files(file_paths: &[impl AsRef<Path>]) -> Result<JsonValue, Box<dyn std::error::Error>> {
    let mut merged_array = Vec::new();

    for path in file_paths {
        let file = File::open(path.as_ref())?;
        let mut reader = BufReader::new(file);
        let mut contents = String::new();
        reader.read_to_string(&mut contents)?;

        let parsed: JsonValue = serde_json::from_str(&contents)?;
        
        if let JsonValue::Array(arr) = parsed {
            merged_array.extend(arr);
        } else {
            merged_array.push(parsed);
        }
    }

    Ok(JsonValue::Array(merged_array))
}

pub fn deduplicate_json_array(array: JsonValue, key: &str) -> JsonValue {
    if let JsonValue::Array(arr) = array {
        let mut seen = HashMap::new();
        let mut deduped = Vec::new();

        for item in arr {
            if let Some(obj) = item.as_object() {
                if let Some(value) = obj.get(key) {
                    let key_str = value.to_string();
                    if !seen.contains_key(&key_str) {
                        seen.insert(key_str.clone(), true);
                        deduped.push(item);
                    }
                } else {
                    deduped.push(item);
                }
            } else {
                deduped.push(item);
            }
        }

        JsonValue::Array(deduped)
    } else {
        array
    }
}

pub fn write_merged_json(output_path: impl AsRef<Path>, data: &JsonValue) -> std::io::Result<()> {
    let json_string = serde_json::to_string_pretty(data)?;
    fs::write(output_path, json_string)
}use serde_json::{Map, Value};
use std::fs;
use std::path::Path;

pub fn merge_json_files(file_paths: &[&str]) -> Result<Value, Box<dyn std::error::Error>> {
    let mut merged_map = Map::new();

    for path_str in file_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            eprintln!("Warning: File {} not found, skipping.", path_str);
            continue;
        }

        let content = fs::read_to_string(path)?;
        let json_value: Value = serde_json::from_str(&content)?;

        if let Value::Object(map) = json_value {
            for (key, value) in map {
                if merged_map.contains_key(&key) {
                    eprintln!("Warning: Key '{}' already exists, overwriting.", key);
                }
                merged_map.insert(key, value);
            }
        } else {
            eprintln!("Warning: File {} does not contain a JSON object, skipping.", path_str);
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

        assert_eq!(obj.get("name").unwrap().as_str().unwrap(), "Alice");
        assert_eq!(obj.get("age").unwrap().as_u64().unwrap(), 30);
        assert_eq!(obj.get("city").unwrap().as_str().unwrap(), "Berlin");
        assert_eq!(obj.get("active").unwrap().as_bool().unwrap(), true);
        assert_eq!(obj.len(), 4);
    }

    #[test]
    fn test_merge_with_overwrite() {
        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();

        writeln!(file1, r#"{"id": 1, "value": "old"}"#).unwrap();
        writeln!(file2, r#"{"id": 2, "value": "new"}"#).unwrap();

        let paths = [
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ];

        let result = merge_json_files(&paths).unwrap();
        let obj = result.as_object().unwrap();

        assert_eq!(obj.get("id").unwrap().as_u64().unwrap(), 2);
        assert_eq!(obj.get("value").unwrap().as_str().unwrap(), "new");
        assert_eq!(obj.len(), 2);
    }
}