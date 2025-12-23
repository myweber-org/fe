
use serde_json::{Value, Map};
use std::fs;
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P]) -> Result<Value, Box<dyn std::error::Error>> {
    let mut merged_map = Map::new();

    for path in paths {
        let content = fs::read_to_string(path)?;
        let json_value: Value = serde_json::from_str(&content)?;

        if let Value::Object(map) = json_value {
            for (key, value) in map {
                if merged_map.contains_key(&key) {
                    let existing_value = merged_map.get(&key).unwrap();
                    if existing_value != &value {
                        let conflict_key = format!("{}_conflict", key);
                        merged_map.insert(conflict_key, value);
                    }
                } else {
                    merged_map.insert(key, value);
                }
            }
        }
    }

    Ok(Value::Object(merged_map))
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

        let json1 = json!({
            "name": "Alice",
            "age": 30
        });

        let json2 = json!({
            "name": "Bob",
            "city": "New York"
        });

        write!(file1, "{}", json1).unwrap();
        write!(file2, "{}", json2).unwrap();

        let result = merge_json_files(&[file1.path(), file2.path()]).unwrap();
        
        assert_eq!(result["name"], "Alice");
        assert_eq!(result["age"], 30);
        assert_eq!(result["city"], "New York");
        assert_eq!(result["name_conflict"], "Bob");
    }
}