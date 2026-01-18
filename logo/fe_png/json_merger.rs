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
}