use serde_json::{Map, Value};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub fn merge_json_files(file_paths: &[&str]) -> Result<Value, Box<dyn std::error::Error>> {
    let mut merged_map = Map::new();

    for path_str in file_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            continue;
        }

        let content = fs::read_to_string(path)?;
        let json_value: Value = serde_json::from_str(&content)?;

        if let Value::Object(obj) = json_value {
            for (key, value) in obj {
                merged_map.insert(key, value);
            }
        }
    }

    Ok(Value::Object(merged_map))
}

pub fn merge_json_with_strategy(
    file_paths: &[&str],
    strategy: MergeStrategy,
) -> Result<Value, Box<dyn std::error::Error>> {
    let mut accumulator: HashMap<String, Value> = HashMap::new();

    for path_str in file_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            continue;
        }

        let content = fs::read_to_string(path)?;
        let json_value: Value = serde_json::from_str(&content)?;

        if let Value::Object(obj) = json_value {
            for (key, value) in obj {
                strategy.apply(&mut accumulator, key, value);
            }
        }
    }

    let final_map: Map<String, Value> = accumulator.into_iter().collect();
    Ok(Value::Object(final_map))
}

pub enum MergeStrategy {
    Overwrite,
    CombineArrays,
    SkipIfExists,
}

impl MergeStrategy {
    fn apply(&self, acc: &mut HashMap<String, Value>, key: String, value: Value) {
        match self {
            MergeStrategy::Overwrite => {
                acc.insert(key, value);
            }
            MergeStrategy::CombineArrays => {
                if let Some(existing) = acc.get_mut(&key) {
                    if let (Value::Array(existing_arr), Value::Array(new_arr)) = (existing, &value) {
                        existing_arr.extend(new_arr.clone());
                    } else {
                        acc.insert(key, value);
                    }
                } else {
                    acc.insert(key, value);
                }
            }
            MergeStrategy::SkipIfExists => {
                acc.entry(key).or_insert(value);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_temp_json(content: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "{}", content).unwrap();
        file
    }

    #[test]
    fn test_merge_basic() {
        let file1 = create_temp_json(r#"{"a": 1, "b": 2}"#);
        let file2 = create_temp_json(r#"{"c": 3, "d": 4}"#);

        let result = merge_json_files(&[
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ])
        .unwrap();

        let expected = json!({
            "a": 1,
            "b": 2,
            "c": 3,
            "d": 4
        });

        assert_eq!(result, expected);
    }

    #[test]
    fn test_strategy_overwrite() {
        let file1 = create_temp_json(r#"{"key": "first"}"#);
        let file2 = create_temp_json(r#"{"key": "second"}"#);

        let result = merge_json_with_strategy(
            &[
                file1.path().to_str().unwrap(),
                file2.path().to_str().unwrap(),
            ],
            MergeStrategy::Overwrite,
        )
        .unwrap();

        assert_eq!(result["key"], "second");
    }

    #[test]
    fn test_strategy_combine_arrays() {
        let file1 = create_temp_json(r#"{"items": [1, 2]}"#);
        let file2 = create_temp_json(r#"{"items": [3, 4]}"#);

        let result = merge_json_with_strategy(
            &[
                file1.path().to_str().unwrap(),
                file2.path().to_str().unwrap(),
            ],
            MergeStrategy::CombineArrays,
        )
        .unwrap();

        let items = result["items"].as_array().unwrap();
        assert_eq!(items.len(), 4);
        assert!(items.contains(&json!(1)));
        assert!(items.contains(&json!(4)));
    }
}