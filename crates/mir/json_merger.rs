
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
}