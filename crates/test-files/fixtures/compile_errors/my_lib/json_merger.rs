
use serde_json::{Map, Value};
use std::collections::HashSet;

pub enum MergeStrategy {
    PreferFirst,
    PreferSecond,
    CombineArrays,
    DeepMerge,
}

pub fn merge_json(
    first: &Map<String, Value>,
    second: &Map<String, Value>,
    strategy: &MergeStrategy,
) -> Map<String, Value> {
    let mut result = first.clone();
    let mut conflicts = HashSet::new();

    for (key, second_value) in second {
        match result.get(key) {
            Some(first_value) => {
                conflicts.insert(key.clone());
                let merged_value = resolve_conflict(first_value, second_value, strategy);
                result.insert(key.clone(), merged_value);
            }
            None => {
                result.insert(key.clone(), second_value.clone());
            }
        }
    }

    if !conflicts.is_empty() {
        log::debug!("Resolved conflicts for keys: {:?}", conflicts);
    }

    result
}

fn resolve_conflict(
    first: &Value,
    second: &Value,
    strategy: &MergeStrategy,
) -> Value {
    match strategy {
        MergeStrategy::PreferFirst => first.clone(),
        MergeStrategy::PreferSecond => second.clone(),
        MergeStrategy::CombineArrays => {
            if let (Value::Array(a), Value::Array(b)) = (first, second) {
                let mut combined = a.clone();
                combined.extend(b.clone());
                Value::Array(combined)
            } else {
                second.clone()
            }
        }
        MergeStrategy::DeepMerge => {
            if let (Value::Object(a), Value::Object(b)) = (first, second) {
                let merged_map = merge_json(a, b, strategy);
                Value::Object(merged_map)
            } else {
                second.clone()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_merge_prefer_first() {
        let mut first = Map::new();
        first.insert("key1".to_string(), json!("value1"));
        first.insert("key2".to_string(), json!(["a", "b"]));

        let mut second = Map::new();
        second.insert("key2".to_string(), json!(["c", "d"]));
        second.insert("key3".to_string(), json!("value3"));

        let merged = merge_json(&first, &second, &MergeStrategy::PreferFirst);

        assert_eq!(merged.get("key1").unwrap(), &json!("value1"));
        assert_eq!(merged.get("key2").unwrap(), &json!(["a", "b"]));
        assert_eq!(merged.get("key3").unwrap(), &json!("value3"));
    }

    #[test]
    fn test_merge_combine_arrays() {
        let mut first = Map::new();
        first.insert("items".to_string(), json!([1, 2]));

        let mut second = Map::new();
        second.insert("items".to_string(), json!([3, 4]));

        let merged = merge_json(&first, &second, &MergeStrategy::CombineArrays);

        if let Value::Array(arr) = merged.get("items").unwrap() {
            assert_eq!(arr.len(), 4);
            assert_eq!(arr, &vec![json!(1), json!(2), json!(3), json!(4)]);
        } else {
            panic!("Expected array");
        }
    }
}