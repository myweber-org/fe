
use serde_json::{Value, Map};
use std::collections::HashSet;

pub enum MergeStrategy {
    PreferFirst,
    PreferSecond,
    CombineArrays,
    FailOnConflict,
}

pub fn merge_json(
    first: &Value,
    second: &Value,
    strategy: &MergeStrategy,
) -> Result<Value, String> {
    match (first, second) {
        (Value::Object(first_map), Value::Object(second_map)) => {
            merge_objects(first_map, second_map, strategy)
        }
        (Value::Array(first_arr), Value::Array(second_arr)) => {
            merge_arrays(first_arr, second_arr, strategy)
        }
        _ => {
            if first == second {
                Ok(first.clone())
            } else {
                handle_scalar_conflict(first, second, strategy)
            }
        }
    }
}

fn merge_objects(
    first: &Map<String, Value>,
    second: &Map<String, Value>,
    strategy: &MergeStrategy,
) -> Result<Value, String> {
    let mut result = Map::new();
    let first_keys: HashSet<_> = first.keys().collect();
    let second_keys: HashSet<_> = second.keys().collect();

    for key in first_keys.union(&second_keys) {
        let key_str = key.to_string();
        let first_val = first.get(&key_str);
        let second_val = second.get(&key_str);

        match (first_val, second_val) {
            (Some(f), Some(s)) => {
                let merged = merge_json(f, s, strategy)?;
                result.insert(key_str, merged);
            }
            (Some(val), None) | (None, Some(val)) => {
                result.insert(key_str, val.clone());
            }
            (None, None) => unreachable!(),
        }
    }

    Ok(Value::Object(result))
}

fn merge_arrays(
    first: &[Value],
    second: &[Value],
    strategy: &MergeStrategy,
) -> Result<Value, String> {
    match strategy {
        MergeStrategy::CombineArrays => {
            let mut combined = Vec::with_capacity(first.len() + second.len());
            combined.extend_from_slice(first);
            combined.extend_from_slice(second);
            Ok(Value::Array(combined))
        }
        _ => handle_scalar_conflict(&Value::Array(first.to_vec()), &Value::Array(second.to_vec()), strategy),
    }
}

fn handle_scalar_conflict(
    first: &Value,
    second: &Value,
    strategy: &MergeStrategy,
) -> Result<Value, String> {
    match strategy {
        MergeStrategy::PreferFirst => Ok(first.clone()),
        MergeStrategy::PreferSecond => Ok(second.clone()),
        MergeStrategy::FailOnConflict => Err(format!(
            "Conflict between values: {} and {}",
            first, second
        )),
        MergeStrategy::CombineArrays => Err("Cannot combine non-array values".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_merge_objects_prefer_first() {
        let first = json!({"a": 1, "b": {"x": 10}});
        let second = json!({"a": 2, "b": {"y": 20}});
        let result = merge_json(&first, &second, &MergeStrategy::PreferFirst).unwrap();
        assert_eq!(result["a"], 1);
        assert_eq!(result["b"]["x"], 10);
        assert!(result["b"].get("y").is_none());
    }

    #[test]
    fn test_merge_arrays_combine() {
        let first = json!([1, 2]);
        let second = json!([3, 4]);
        let result = merge_json(&first, &second, &MergeStrategy::CombineArrays).unwrap();
        assert_eq!(result, json!([1, 2, 3, 4]));
    }
}