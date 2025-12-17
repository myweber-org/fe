use serde_json::{Map, Value};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub fn merge_json_files(file_paths: &[&str]) -> Result<Value, Box<dyn std::error::Error>> {
    let mut merged_map = Map::new();

    for &file_path in file_paths {
        if !Path::new(file_path).exists() {
            eprintln!("Warning: File {} not found, skipping.", file_path);
            continue;
        }

        let content = fs::read_to_string(file_path)?;
        let json_value: Value = serde_json::from_str(&content)?;

        if let Value::Object(map) = json_value {
            for (key, value) in map {
                merged_map.insert(key, value);
            }
        } else {
            eprintln!("Warning: File {} does not contain a JSON object, skipping.", file_path);
        }
    }

    Ok(Value::Object(merged_map))
}

pub fn merge_json_with_strategy(
    file_paths: &[&str],
    conflict_strategy: fn(&str, &Value, &Value) -> Value,
) -> Result<Value, Box<dyn std::error::Error>> {
    let mut accumulator: HashMap<String, Value> = HashMap::new();

    for &file_path in file_paths {
        if !Path::new(file_path).exists() {
            eprintln!("Warning: File {} not found, skipping.", file_path);
            continue;
        }

        let content = fs::read_to_string(file_path)?;
        let json_value: Value = serde_json::from_str(&content)?;

        if let Value::Object(map) = json_value {
            for (key, value) in map {
                match accumulator.get(&key) {
                    Some(existing_value) => {
                        let resolved_value = conflict_strategy(&key, existing_value, &value);
                        accumulator.insert(key, resolved_value);
                    }
                    None => {
                        accumulator.insert(key, value);
                    }
                }
            }
        } else {
            eprintln!("Warning: File {} does not contain a JSON object, skipping.", file_path);
        }
    }

    let final_map: Map<String, Value> = accumulator.into_iter().collect();
    Ok(Value::Object(final_map))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_files() {
        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();

        let data1 = json!({"name": "Alice", "age": 30});
        let data2 = json!({"city": "Berlin", "country": "Germany"});

        write!(file1, "{}", data1.to_string()).unwrap();
        write!(file2, "{}", data2.to_string()).unwrap();

        let paths = vec![
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ];

        let result = merge_json_files(&paths).unwrap();
        let expected = json!({
            "name": "Alice",
            "age": 30,
            "city": "Berlin",
            "country": "Germany"
        });

        assert_eq!(result, expected);
    }

    #[test]
    fn test_merge_with_conflict_strategy() {
        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();

        let data1 = json!({"score": 100});
        let data2 = json!({"score": 200});

        write!(file1, "{}", data1.to_string()).unwrap();
        write!(file2, "{}", data2.to_string()).unwrap();

        let paths = vec![
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ];

        let strategy = |_key: &str, v1: &Value, v2: &Value| {
            let n1 = v1.as_i64().unwrap_or(0);
            let n2 = v2.as_i64().unwrap_or(0);
            json!(n1.max(n2))
        };

        let result = merge_json_with_strategy(&paths, strategy).unwrap();
        assert_eq!(result, json!({"score": 200}));
    }
}