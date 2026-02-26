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
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::Path;

pub fn merge_json_files(input_paths: &[&str], output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged_data = HashMap::new();

    for path_str in input_paths {
        let path = Path::new(path_str);
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let data: HashMap<String, serde_json::Value> = serde_json::from_reader(reader)?;

        for (key, value) in data {
            merged_data.insert(key, value);
        }
    }

    let output_file = File::create(output_path)?;
    serde_json::to_writer_pretty(output_file, &merged_data)?;

    Ok(())
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

        let output_file = NamedTempFile::new().unwrap();
        let output_path = output_file.path().to_str().unwrap();

        let input_paths = vec![
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ];

        let result = merge_json_files(&input_paths, output_path);
        assert!(result.is_ok());

        let output_content = std::fs::read_to_string(output_path).unwrap();
        let parsed: HashMap<String, serde_json::Value> = serde_json::from_str(&output_content).unwrap();

        assert_eq!(parsed.get("name").unwrap(), "Alice");
        assert_eq!(parsed.get("age").unwrap(), 30);
        assert_eq!(parsed.get("city").unwrap(), "Berlin");
        assert_eq!(parsed.get("active").unwrap(), true);
    }
}