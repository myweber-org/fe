use serde_json::{Map, Value};
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

        if let Value::Object(map) = json_value {
            for (key, value) in map {
                merged_map.insert(key, value);
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

        let data1 = json!({
            "name": "test",
            "count": 42
        });
        let data2 = json!({
            "enabled": true,
            "tags": ["rust", "json"]
        });

        write!(file1, "{}", data1.to_string()).unwrap();
        write!(file2, "{}", data2.to_string()).unwrap();

        let paths = vec![
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
            "non_existent.json"
        ];

        let result = merge_json_files(&paths).unwrap();
        let expected = json!({
            "name": "test",
            "count": 42,
            "enabled": true,
            "tags": ["rust", "json"]
        });

        assert_eq!(result, expected);
    }
}
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

type JsonValue = serde_json::Value;
type JsonMap = HashMap<String, JsonValue>;

pub fn merge_json_files(file_paths: &[impl AsRef<Path>]) -> Result<JsonValue, Box<dyn std::error::Error>> {
    let mut merged_map = JsonMap::new();

    for path in file_paths {
        let file = File::open(path.as_ref())?;
        let mut reader = BufReader::new(file);
        let mut contents = String::new();
        reader.read_to_string(&mut contents)?;

        let json_value: JsonValue = serde_json::from_str(&contents)?;

        if let JsonValue::Object(map) = json_value {
            for (key, value) in map {
                merged_map.insert(key, value);
            }
        } else {
            return Err("Each JSON file must contain a JSON object".into());
        }
    }

    Ok(JsonValue::Object(serde_json::Map::from_iter(merged_map)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_files() {
        let json1 = r#"{"name": "Alice", "age": 30}"#;
        let json2 = r#"{"city": "Berlin", "active": true}"#;

        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();

        file1.write_all(json1.as_bytes()).unwrap();
        file2.write_all(json2.as_bytes()).unwrap();

        let result = merge_json_files(&[file1.path(), file2.path()]).unwrap();
        let result_obj = result.as_object().unwrap();

        assert_eq!(result_obj.get("name").unwrap(), "Alice");
        assert_eq!(result_obj.get("age").unwrap(), 30);
        assert_eq!(result_obj.get("city").unwrap(), "Berlin");
        assert_eq!(result_obj.get("active").unwrap(), true);
    }

    #[test]
    fn test_merge_with_override() {
        let json1 = r#"{"id": 1, "value": "original"}"#;
        let json2 = r#"{"id": 2, "value": "updated"}"#;

        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();

        file1.write_all(json1.as_bytes()).unwrap();
        file2.write_all(json2.as_bytes()).unwrap();

        let result = merge_json_files(&[file1.path(), file2.path()]).unwrap();
        let result_obj = result.as_object().unwrap();

        assert_eq!(result_obj.get("id").unwrap(), 2);
        assert_eq!(result_obj.get("value").unwrap(), "updated");
    }
}