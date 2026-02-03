
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
    let mut result = first.clone();
    let mut conflicts = Vec::new();

    for (key, value2) in second {
        match result.get(key) {
            Some(value1) => {
                if value1 != value2 {
                    match resolution {
                        ConflictResolution::PreferFirst => continue,
                        ConflictResolution::PreferSecond => {
                            result.insert(key.clone(), value2.clone());
                        }
                        ConflictResolution::MergeArrays => {
                            if let (Value::Array(arr1), Value::Array(arr2)) = (value1, value2) {
                                let mut merged = arr1.clone();
                                merged.extend(arr2.clone());
                                result.insert(key.clone(), Value::Array(merged));
                            } else {
                                conflicts.push(key.clone());
                            }
                        }
                        ConflictResolution::FailOnConflict => {
                            return Err(format!("Conflict detected on key: {}", key));
                        }
                    }
                }
            }
            None => {
                result.insert(key.clone(), value2.clone());
            }
        }
    }

    if !conflicts.is_empty() && matches!(resolution, ConflictResolution::MergeArrays) {
        return Err(format!(
            "Cannot merge non-array values for keys: {:?}",
            conflicts
        ));
    }

    Ok(result)
}

pub fn find_common_keys(first: &Map<String, Value>, second: &Map<String, Value>) -> HashSet<String> {
    first.keys().filter(|k| second.contains_key(*k)).cloned().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_merge_prefer_first() {
        let mut map1 = Map::new();
        map1.insert("a".to_string(), json!(1));
        map1.insert("b".to_string(), json!("test"));

        let mut map2 = Map::new();
        map2.insert("a".to_string(), json!(2));
        map2.insert("c".to_string(), json!(true));

        let merged = merge_json(&map1, &map2, ConflictResolution::PreferFirst).unwrap();
        assert_eq!(merged.get("a"), Some(&json!(1)));
        assert_eq!(merged.get("c"), Some(&json!(true)));
    }

    #[test]
    fn test_merge_arrays() {
        let mut map1 = Map::new();
        map1.insert("items".to_string(), json!([1, 2]));

        let mut map2 = Map::new();
        map2.insert("items".to_string(), json!([3, 4]));

        let merged = merge_json(&map1, &map2, ConflictResolution::MergeArrays).unwrap();
        assert_eq!(merged.get("items"), Some(&json!([1, 2, 3, 4])));
    }
}