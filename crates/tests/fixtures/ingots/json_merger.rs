use std::collections::HashMap;
use serde_json::{Value, Map};

pub fn merge_json(base: &mut Value, extension: &Value) {
    match (base, extension) {
        (Value::Object(base_map), Value::Object(ext_map)) => {
            for (key, ext_value) in ext_map {
                if let Some(base_value) = base_map.get_mut(key) {
                    merge_json(base_value, ext_value);
                } else {
                    base_map.insert(key.clone(), ext_value.clone());
                }
            }
        }
        (base, extension) => {
            *base = extension.clone();
        }
    }
}

pub fn merge_multiple_json(objects: Vec<Value>) -> Option<Value> {
    let mut result = Value::Object(Map::new());
    for obj in objects {
        merge_json(&mut result, &obj);
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
        let extension = json!({"b": {"d": 3}, "e": 4});
        merge_json(&mut base, &extension);
        assert_eq!(base, json!({"a": 1, "b": {"c": 2, "d": 3}, "e": 4}));
    }

    #[test]
    fn test_overwrite_primitive() {
        let mut base = json!({"a": 1});
        let extension = json!({"a": 2});
        merge_json(&mut base, &extension);
        assert_eq!(base, json!({"a": 2}));
    }

    #[test]
    fn test_multiple_merge() {
        let objects = vec![
            json!({"a": 1}),
            json!({"b": 2}),
            json!({"a": 3, "c": 4})
        ];
        let result = merge_multiple_json(objects).unwrap();
        assert_eq!(result, json!({"a": 3, "b": 2, "c": 4}));
    }
}