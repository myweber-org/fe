
use std::collections::HashMap;
use std::fs;
use std::path::Path;

type JsonValue = serde_json::Value;

pub fn merge_json_files(file_paths: &[impl AsRef<Path>]) -> Result<JsonValue, Box<dyn std::error::Error>> {
    let mut merged_map = HashMap::new();

    for path in file_paths {
        let content = fs::read_to_string(path)?;
        let json_data: JsonValue = serde_json::from_str(&content)?;

        if let JsonValue::Object(map) = json_data {
            for (key, value) in map {
                merged_map.insert(key, value);
            }
        } else {
            return Err("Each JSON file must contain an object".into());
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

        fs::write(&file1, r#"{"a": 1, "b": 2}"#).unwrap();
        fs::write(&file2, r#"{"c": 3, "d": 4}"#).unwrap();

        let result = merge_json_files(&[file1.path(), file2.path()]).unwrap();
        let expected: JsonValue = serde_json::from_str(r#"{"a":1,"b":2,"c":3,"d":4}"#).unwrap();

        assert_eq!(result, expected);
    }
}