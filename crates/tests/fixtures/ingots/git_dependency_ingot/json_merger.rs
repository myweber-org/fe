use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, Read};
use std::path::Path;

type JsonValue = serde_json::Value;

pub fn merge_json_files(file_paths: &[impl AsRef<Path>]) -> Result<JsonValue, Box<dyn std::error::Error>> {
    let mut merged_array = Vec::new();

    for path in file_paths {
        let file = File::open(path.as_ref())?;
        let mut reader = BufReader::new(file);
        let mut contents = String::new();
        reader.read_to_string(&mut contents)?;

        let json_value: JsonValue = serde_json::from_str(&contents)?;
        
        match json_value {
            JsonValue::Array(arr) => {
                merged_array.extend(arr);
            }
            _ => {
                merged_array.push(json_value);
            }
        }
    }

    Ok(JsonValue::Array(merged_array))
}

pub fn merge_json_with_key(
    file_paths: &[impl AsRef<Path>],
    key: &str,
) -> Result<JsonValue, Box<dyn std::error::Error>> {
    let mut merged_map = HashMap::new();

    for path in file_paths {
        let contents = fs::read_to_string(path.as_ref())?;
        let json_value: JsonValue = serde_json::from_str(&contents)?;

        if let JsonValue::Object(obj) = json_value {
            if let Some(value) = obj.get(key) {
                merged_map.insert(key.to_string(), value.clone());
            }
        }
    }

    Ok(JsonValue::Object(serde_json::Map::from_iter(merged_map)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_files() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();

        fs::write(&file1, r#"[{"id": 1}, {"id": 2}]"#).unwrap();
        fs::write(&file2, r#"{"id": 3}"#).unwrap();

        let result = merge_json_files(&[file1.path(), file2.path()]).unwrap();
        assert!(result.is_array());
        assert_eq!(result.as_array().unwrap().len(), 3);
    }

    #[test]
    fn test_merge_json_with_key() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();

        fs::write(&file1, r#"{"user": "alice", "age": 30}"#).unwrap();
        fs::write(&file2, r#"{"user": "bob", "age": 25}"#).unwrap();

        let result = merge_json_with_key(&[file1.path(), file2.path()], "user").unwrap();
        assert!(result.is_object());
        let obj = result.as_object().unwrap();
        assert!(obj.contains_key("user"));
    }
}