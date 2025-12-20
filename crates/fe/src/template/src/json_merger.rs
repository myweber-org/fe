
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, Read, Write};
use std::path::Path;

use serde_json::{json, Value};

pub fn merge_json_files(input_paths: &[String], output_path: &str) -> Result<(), String> {
    let mut merged_array = Vec::new();

    for path_str in input_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            return Err(format!("File not found: {}", path_str));
        }

        let file = File::open(path).map_err(|e| e.to_string())?;
        let mut reader = BufReader::new(file);
        let mut contents = String::new();
        reader.read_to_string(&mut contents).map_err(|e| e.to_string())?;

        let json_value: Value = serde_json::from_str(&contents).map_err(|e| e.to_string())?;
        merged_array.push(json_value);
    }

    let merged_json = json!(merged_array);
    let pretty_json = serde_json::to_string_pretty(&merged_json).map_err(|e| e.to_string())?;

    let mut output_file = File::create(output_path).map_err(|e| e.to_string())?;
    output_file
        .write_all(pretty_json.as_bytes())
        .map_err(|e| e.to_string())?;

    Ok(())
}

pub fn merge_json_with_deduplication(
    input_paths: &[String],
    output_path: &str,
    key_field: &str,
) -> Result<(), String> {
    let mut unique_map: HashMap<String, Value> = HashMap::new();

    for path_str in input_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            return Err(format!("File not found: {}", path_str));
        }

        let contents = fs::read_to_string(path).map_err(|e| e.to_string())?;
        let json_value: Value = serde_json::from_str(&contents).map_err(|e| e.to_string())?;

        match json_value {
            Value::Array(arr) => {
                for item in arr {
                    if let Some(obj) = item.as_object() {
                        if let Some(key_value) = obj.get(key_field) {
                            if let Some(key_str) = key_value.as_str() {
                                unique_map.insert(key_str.to_string(), item.clone());
                            }
                        }
                    }
                }
            }
            _ => return Err("Expected JSON array in input files".to_string()),
        }
    }

    let deduplicated_array: Vec<Value> = unique_map.into_values().collect();
    let merged_json = json!(deduplicated_array);
    let pretty_json = serde_json::to_string_pretty(&merged_json).map_err(|e| e.to_string())?;

    fs::write(output_path, pretty_json).map_err(|e| e.to_string())?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_basic_merge() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();

        fs::write(
            file1.path(),
            r#"[{"id": 1, "name": "Alice"}, {"id": 2, "name": "Bob"}]"#,
        )
        .unwrap();
        fs::write(file2.path(), r#"[{"id": 3, "name": "Charlie"}]"#).unwrap();

        let inputs = vec![
            file1.path().to_str().unwrap().to_string(),
            file2.path().to_str().unwrap().to_string(),
        ];

        let result = merge_json_files(&inputs, output_file.path().to_str().unwrap());
        assert!(result.is_ok());

        let output_content = fs::read_to_string(output_file.path()).unwrap();
        let parsed: Value = serde_json::from_str(&output_content).unwrap();
        assert!(parsed.is_array());
        assert_eq!(parsed.as_array().unwrap().len(), 3);
    }

    #[test]
    fn test_deduplication() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();

        fs::write(
            file1.path(),
            r#"[{"id": "a1", "value": 100}, {"id": "a2", "value": 200}]"#,
        )
        .unwrap();
        fs::write(
            file2.path(),
            r#"[{"id": "a2", "value": 250}, {"id": "a3", "value": 300}]"#,
        )
        .unwrap();

        let inputs = vec![
            file1.path().to_str().unwrap().to_string(),
            file2.path().to_str().unwrap().to_string(),
        ];

        let result = merge_json_with_deduplication(
            &inputs,
            output_file.path().to_str().unwrap(),
            "id",
        );
        assert!(result.is_ok());

        let output_content = fs::read_to_string(output_file.path()).unwrap();
        let parsed: Value = serde_json::from_str(&output_content).unwrap();
        let array = parsed.as_array().unwrap();
        assert_eq!(array.len(), 3);

        let ids: Vec<&str> = array
            .iter()
            .filter_map(|v| v.as_object().unwrap().get("id").unwrap().as_str())
            .collect();
        assert!(ids.contains(&"a1"));
        assert!(ids.contains(&"a2"));
        assert!(ids.contains(&"a3"));
    }
}