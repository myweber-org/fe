
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, BufReader, Write};
use std::path::Path;

use serde_json::{json, Value};

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P], output_path: P) -> io::Result<()> {
    let mut merged_array = Vec::new();

    for path in paths {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let json_value: Value = serde_json::from_reader(reader)?;

        match json_value {
            Value::Array(arr) => {
                merged_array.extend(arr);
            }
            Value::Object(obj) => {
                merged_array.push(Value::Object(obj));
            }
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "JSON must be an array or object",
                ));
            }
        }
    }

    let output_file = File::create(output_path)?;
    serde_json::to_writer_pretty(output_file, &json!(merged_array))?;

    Ok(())
}

pub fn merge_json_with_deduplication<P: AsRef<Path>>(
    paths: &[P],
    output_path: P,
    key_field: &str,
) -> io::Result<()> {
    let mut unique_map: HashMap<String, Value> = HashMap::new();

    for path in paths {
        let content = fs::read_to_string(path)?;
        let json_value: Value = serde_json::from_str(&content)?;

        let items = match json_value {
            Value::Array(arr) => arr,
            Value::Object(obj) => vec![Value::Object(obj)],
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "JSON must be an array or object",
                ));
            }
        };

        for item in items {
            if let Value::Object(map) = item {
                if let Some(key_value) = map.get(key_field) {
                    let key = key_value.to_string();
                    unique_map.insert(key, Value::Object(map));
                }
            }
        }
    }

    let deduplicated_array: Vec<Value> = unique_map.into_values().collect();
    let output_file = File::create(output_path)?;
    serde_json::to_writer_pretty(output_file, &json!(deduplicated_array))?;

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

        let data1 = r#"[{"id": 1, "name": "Alice"}, {"id": 2, "name": "Bob"}]"#;
        let data2 = r#"[{"id": 3, "name": "Charlie"}]"#;

        fs::write(&file1, data1).unwrap();
        fs::write(&file2, data2).unwrap();

        merge_json_files(&[file1.path(), file2.path()], output_file.path()).unwrap();

        let output_content = fs::read_to_string(output_file.path()).unwrap();
        let parsed: Value = serde_json::from_str(&output_content).unwrap();

        assert!(parsed.is_array());
        let array = parsed.as_array().unwrap();
        assert_eq!(array.len(), 3);
    }
}