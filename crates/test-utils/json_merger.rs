
use std::collections::HashMap;
use serde_json::{Value, Map};

pub fn merge_json(base: &mut Value, update: &Value, resolve_conflict: bool) -> Result<(), String> {
    match (base, update) {
        (Value::Object(base_map), Value::Object(update_map)) => {
            for (key, update_value) in update_map {
                if base_map.contains_key(key) {
                    let base_value = base_map.get_mut(key).unwrap();
                    if base_value.is_object() && update_value.is_object() {
                        merge_json(base_value, update_value, resolve_conflict)?;
                    } else if resolve_conflict {
                        *base_value = update_value.clone();
                    } else {
                        return Err(format!("Conflict detected for key: {}", key));
                    }
                } else {
                    base_map.insert(key.clone(), update_value.clone());
                }
            }
            Ok(())
        }
        _ => Err("Both inputs must be JSON objects".to_string())
    }
}

pub fn merge_multiple_json(objects: Vec<Value>, resolve_conflict: bool) -> Result<Value, String> {
    if objects.is_empty() {
        return Ok(Value::Object(Map::new()));
    }

    let mut result = objects[0].clone();
    for obj in objects.iter().skip(1) {
        merge_json(&mut result, obj, resolve_conflict)?;
    }
    Ok(result)
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
    fn test_merge_multiple_objects() {
        let objects = vec![
            json!({"a": 1}),
            json!({"b": 2}),
            json!({"c": 3})
        ];
        
        let result = merge_multiple_json(objects, false).unwrap();
        assert_eq!(result, json!({"a": 1, "b": 2, "c": 3}));
    }
}