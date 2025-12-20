
use serde_json::{Map, Value};
use std::collections::HashSet;

pub enum ConflictResolution {
    PreferFirst,
    PreferSecond,
    MergeArrays,
    FailOnConflict,
}

pub fn merge_json(
    first: &Map<String, Value>,
    second: &Map<String, Value>,
    resolution: ConflictResolution,
) -> Result<Map<String, Value>, String> {
    let mut result = Map::new();
    let mut all_keys: HashSet<&String> = first.keys().chain(second.keys()).collect();

    for key in all_keys {
        let first_val = first.get(key);
        let second_val = second.get(key);

        match (first_val, second_val) {
            (Some(f), None) => {
                result.insert(key.clone(), f.clone());
            }
            (None, Some(s)) => {
                result.insert(key.clone(), s.clone());
            }
            (Some(f), Some(s)) => {
                let merged = handle_conflict(key, f, s, &resolution)?;
                result.insert(key.clone(), merged);
            }
            _ => unreachable!(),
        }
    }

    Ok(result)
}

fn handle_conflict(
    key: &str,
    first: &Value,
    second: &Value,
    resolution: &ConflictResolution,
) -> Result<Value, String> {
    match resolution {
        ConflictResolution::PreferFirst => Ok(first.clone()),
        ConflictResolution::PreferSecond => Ok(second.clone()),
        ConflictResolution::MergeArrays => {
            if let (Value::Array(a1), Value::Array(a2)) = (first, second) {
                let mut merged = a1.clone();
                merged.extend(a2.clone());
                Ok(Value::Array(merged))
            } else {
                Err(format!(
                    "Cannot merge non-array values for key '{}' with MergeArrays strategy",
                    key
                ))
            }
        }
        ConflictResolution::FailOnConflict => Err(format!("Conflict detected for key '{}'", key)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_merge_prefer_first() {
        let mut first = Map::new();
        first.insert("a".to_string(), json!(1));
        first.insert("b".to_string(), json!("test"));

        let mut second = Map::new();
        second.insert("b".to_string(), json!("overridden"));
        second.insert("c".to_string(), json!(true));

        let merged = merge_json(&first, &second, ConflictResolution::PreferFirst).unwrap();

        assert_eq!(merged.get("a"), Some(&json!(1)));
        assert_eq!(merged.get("b"), Some(&json!("test")));
        assert_eq!(merged.get("c"), Some(&json!(true)));
    }

    #[test]
    fn test_merge_arrays() {
        let mut first = Map::new();
        first.insert("items".to_string(), json!([1, 2, 3]));

        let mut second = Map::new();
        second.insert("items".to_string(), json!([4, 5]));

        let merged = merge_json(&first, &second, ConflictResolution::MergeArrays).unwrap();

        if let Value::Array(arr) = merged.get("items").unwrap() {
            assert_eq!(arr.len(), 5);
            assert_eq!(arr, &vec![json!(1), json!(2), json!(3), json!(4), json!(5)]);
        } else {
            panic!("Expected array");
        }
    }
}