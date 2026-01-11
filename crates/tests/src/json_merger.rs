use serde_json::{Map, Value};
use std::fs;
use std::path::Path;

pub fn merge_json_files(file_paths: &[&str]) -> Result<Value, Box<dyn std::error::Error>> {
    let mut merged_map = Map::new();

    for file_path in file_paths {
        let path = Path::new(file_path);
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
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_files() {
        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();

        writeln!(file1, r#"{"name": "Alice", "age": 30}"#).unwrap();
        writeln!(file2, r#"{"city": "London", "active": true}"#).unwrap();

        let result = merge_json_files(&[
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ])
        .unwrap();

        assert_eq!(result["name"], "Alice");
        assert_eq!(result["age"], 30);
        assert_eq!(result["city"], "London");
        assert_eq!(result["active"], true);
    }

    #[test]
    fn test_merge_with_missing_file() {
        let mut file1 = NamedTempFile::new().unwrap();
        writeln!(file1, r#"{"data": "test"}"#).unwrap();

        let result = merge_json_files(&[
            file1.path().to_str().unwrap(),
            "non_existent_file.json",
        ])
        .unwrap();

        assert_eq!(result["data"], "test");
    }
}
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Write};
use serde_json::{Value, Map};

fn merge_json_files(file_paths: &[&str], output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged_map: Map<String, Value> = Map::new();

    for path in file_paths {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let json_data: Value = serde_json::from_reader(reader)?;

        if let Value::Object(map) = json_data {
            for (key, value) in map {
                merged_map.insert(key, value);
            }
        } else {
            return Err("Each JSON file must contain a JSON object".into());
        }
    }

    let output_file = File::create(output_path)?;
    serde_json::to_writer_pretty(output_file, &Value::Object(merged_map))?;
    Ok(())
}

fn main() {
    let files_to_merge = vec!["data1.json", "data2.json", "data3.json"];
    let output_file = "merged_output.json";

    match merge_json_files(&files_to_merge, output_file) {
        Ok(_) => println!("Successfully merged JSON files into {}", output_file),
        Err(e) => eprintln!("Error merging JSON files: {}", e),
    }
}