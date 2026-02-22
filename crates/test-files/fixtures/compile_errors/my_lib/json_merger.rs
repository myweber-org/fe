
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
}use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub fn merge_json_files(file_paths: &[&str]) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let mut merged_map = HashMap::new();

    for path_str in file_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            continue;
        }

        let content = fs::read_to_string(path)?;
        let json_value: serde_json::Value = serde_json::from_str(&content)?;

        if let serde_json::Value::Object(map) = json_value {
            for (key, value) in map {
                merged_map.insert(key, value);
            }
        }
    }

    Ok(serde_json::Value::Object(merged_map))
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

        writeln!(file1, r#"{"a": 1, "b": "test"}"#).unwrap();
        writeln!(file2, r#"{"c": true, "d": [1,2,3]}"#).unwrap();

        let paths = [
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ];

        let result = merge_json_files(&paths).unwrap();
        let obj = result.as_object().unwrap();

        assert_eq!(obj.get("a").unwrap(), &serde_json::json!(1));
        assert_eq!(obj.get("b").unwrap(), &serde_json::json!("test"));
        assert_eq!(obj.get("c").unwrap(), &serde_json::json!(true));
        assert_eq!(obj.get("d").unwrap(), &serde_json::json!([1,2,3]));
    }
}