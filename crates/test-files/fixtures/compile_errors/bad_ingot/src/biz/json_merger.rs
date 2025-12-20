use serde_json::{json, Value};
use std::fs;
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P], output_path: P) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged_array = Vec::new();

    for path in paths {
        let content = fs::read_to_string(path)?;
        let parsed: Value = serde_json::from_str(&content)?;
        merged_array.push(parsed);
    }

    let output_value = Value::Array(merged_array);
    let output_json = serde_json::to_string_pretty(&output_value)?;
    fs::write(output_path, output_json)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_files() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();

        fs::write(&file1, r#"{"id": 1, "name": "test1"}"#).unwrap();
        fs::write(&file2, r#"{"id": 2, "name": "test2"}"#).unwrap();

        let paths = [file1.path(), file2.path()];
        merge_json_files(&paths, output_file.path()).unwrap();

        let output_content = fs::read_to_string(output_file.path()).unwrap();
        let expected = r#"[
  {
    "id": 1,
    "name": "test1"
  },
  {
    "id": 2,
    "name": "test2"
  }
]"#;
        assert_eq!(output_content.trim(), expected.trim());
    }
}
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
                merge_value(&mut merged_map, key, value);
            }
        }
    }

    Ok(Value::Object(merged_map))
}

fn merge_value(map: &mut Map<String, Value>, key: String, new_value: Value) {
    match map.get_mut(&key) {
        Some(existing_value) => {
            if let (Value::Object(existing_obj), Value::Object(new_obj)) = (existing_value, &new_value) {
                let mut existing_obj = existing_obj.clone();
                for (nested_key, nested_value) in new_obj {
                    merge_value(&mut existing_obj, nested_key.clone(), nested_value.clone());
                }
                map.insert(key, Value::Object(existing_obj));
            } else if existing_value.is_array() && new_value.is_array() {
                if let (Value::Array(existing_arr), Value::Array(new_arr)) = (existing_value, new_value) {
                    existing_arr.extend(new_arr);
                }
            } else {
                map.insert(key, new_value);
            }
        }
        None => {
            map.insert(key, new_value);
        }
    }
}