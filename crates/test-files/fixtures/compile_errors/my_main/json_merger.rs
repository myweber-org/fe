use serde_json::{Value, json};
use std::fs::{self, File};
use std::io::{BufReader, BufWriter};
use std::path::Path;

pub fn merge_json_files(input_paths: &[&str], output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged_array = Vec::new();

    for path_str in input_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            eprintln!("Warning: File {} not found, skipping.", path_str);
            continue;
        }

        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let json_value: Value = serde_json::from_reader(reader)?;

        match json_value {
            Value::Array(arr) => {
                merged_array.extend(arr);
            }
            Value::Object(_) => {
                merged_array.push(json_value);
            }
            _ => {
                eprintln!("Warning: File {} does not contain a JSON object or array, skipping.", path_str);
            }
        }
    }

    let output_file = File::create(output_path)?;
    let writer = BufWriter::new(output_file);
    serde_json::to_writer_pretty(writer, &json!(merged_array))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_files() {
        let json1 = r#"[{"id": 1}, {"id": 2}]"#;
        let json2 = r#"{"id": 3}"#;

        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();
        file1.write_all(json1.as_bytes()).unwrap();
        file2.write_all(json2.as_bytes()).unwrap();

        let output_file = NamedTempFile::new().unwrap();
        let input_paths = vec![
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ];

        let result = merge_json_files(&input_paths, output_file.path().to_str().unwrap());
        assert!(result.is_ok());

        let output_content = fs::read_to_string(output_file.path()).unwrap();
        let parsed: Value = serde_json::from_str(&output_content).unwrap();
        assert_eq!(parsed.as_array().unwrap().len(), 3);
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

    Ok(serde_json::Value::Object(serde_json::Map::from_iter(merged_map)))
}