
use std::collections::HashMap;
use serde_json::{Value, Map};

pub fn merge_json(base: &mut Value, update: &Value, resolve_conflict: bool) -> Result<(), String> {
    match (base, update) {
        (Value::Object(base_map), Value::Object(update_map)) => {
            merge_objects(base_map, update_map, resolve_conflict)
        }
        _ => {
            if resolve_conflict {
                *base = update.clone();
                Ok(())
            } else {
                Err("Type mismatch and conflict resolution disabled".to_string())
            }
        }
    }
}

fn merge_objects(
    base: &mut Map<String, Value>,
    update: &Map<String, Value>,
    resolve_conflict: bool,
) -> Result<(), String> {
    for (key, update_value) in update {
        match base.get_mut(key) {
            Some(base_value) => {
                if let (Value::Object(_), Value::Object(_)) = (base_value, update_value) {
                    merge_json(base_value, update_value, resolve_conflict)?;
                } else if resolve_conflict {
                    *base_value = update_value.clone();
                } else {
                    return Err(format!("Conflict on key '{}'", key));
                }
            }
            None => {
                base.insert(key.clone(), update_value.clone());
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_merge_without_conflict() {
        let mut base = json!({"a": 1, "b": {"c": 2}});
        let update = json!({"b": {"d": 3}, "e": 4});
        
        merge_json(&mut base, &update, false).unwrap();
        assert_eq!(base, json!({"a": 1, "b": {"c": 2, "d": 3}, "e": 4}));
    }

    #[test]
    fn test_merge_with_conflict_resolution() {
        let mut base = json!({"a": 1, "b": 2});
        let update = json!({"b": 3, "c": 4});
        
        merge_json(&mut base, &update, true).unwrap();
        assert_eq!(base, json!({"a": 1, "b": 3, "c": 4}));
    }

    #[test]
    fn test_merge_conflict_error() {
        let mut base = json!({"a": 1});
        let update = json!({"a": {"b": 2}});
        
        let result = merge_json(&mut base, &update, false);
        assert!(result.is_err());
    }
}use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, Read};
use std::path::Path;

type JsonValue = serde_json::Value;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P]) -> Result<JsonValue, String> {
    let mut merged_array = Vec::new();

    for path in paths {
        let file = File::open(path).map_err(|e| format!("Failed to open {}: {}", path.as_ref().display(), e))?;
        let mut reader = BufReader::new(file);
        let mut content = String::new();
        reader.read_to_string(&mut content).map_err(|e| format!("Failed to read {}: {}", path.as_ref().display(), e))?;

        let json_value: JsonValue = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse JSON from {}: {}", path.as_ref().display(), e))?;

        match json_value {
            JsonValue::Array(arr) => merged_array.extend(arr),
            JsonValue::Object(obj) => merged_array.push(JsonValue::Object(obj)),
            _ => return Err(format!("JSON root must be array or object in {}", path.as_ref().display())),
        }
    }

    Ok(JsonValue::Array(merged_array))
}

pub fn merge_json_with_key<P: AsRef<Path>>(paths: &[P], key_field: &str) -> Result<JsonValue, String> {
    let mut merged_map = HashMap::new();

    for path in paths {
        let file = File::open(path).map_err(|e| format!("Failed to open {}: {}", path.as_ref().display(), e))?;
        let mut reader = BufReader::new(file);
        let mut content = String::new();
        reader.read_to_string(&mut content).map_err(|e| format!("Failed to read {}: {}", path.as_ref().display(), e))?;

        let json_value: JsonValue = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse JSON from {}: {}", path.as_ref().display(), e))?;

        match json_value {
            JsonValue::Array(arr) => {
                for item in arr {
                    if let JsonValue::Object(mut obj) = item {
                        if let Some(key_value) = obj.get(key_field) {
                            let key = key_value.to_string();
                            merged_map.insert(key, JsonValue::Object(obj));
                        } else {
                            return Err(format!("Missing key field '{}' in object from {}", key_field, path.as_ref().display()));
                        }
                    }
                }
            }
            JsonValue::Object(obj) => {
                if let Some(key_value) = obj.get(key_field) {
                    let key = key_value.to_string();
                    merged_map.insert(key, JsonValue::Object(obj));
                } else {
                    return Err(format!("Missing key field '{}' in object from {}", key_field, path.as_ref().display()));
                }
            }
            _ => return Err(format!("JSON root must be array or object in {}", path.as_ref().display())),
        }
    }

    let result_array: Vec<JsonValue> = merged_map.into_values().collect();
    Ok(JsonValue::Array(result_array))
}

pub fn write_merged_json<P: AsRef<Path>>(output_path: P, json_value: &JsonValue) -> Result<(), String> {
    let json_string = serde_json::to_string_pretty(json_value)
        .map_err(|e| format!("Failed to serialize JSON: {}", e))?;
    
    fs::write(&output_path, json_string)
        .map_err(|e| format!("Failed to write to {}: {}", output_path.as_ref().display(), e))?;
    
    Ok(())
}