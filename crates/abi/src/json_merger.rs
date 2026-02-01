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
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_files() {
        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();

        writeln!(file1, r#"{"name": "Alice", "age": 30}"#).unwrap();
        writeln!(file2, r#"{"city": "Berlin", "active": true}"#).unwrap();

        let result = merge_json_files(&[
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ])
        .unwrap();

        assert_eq!(result["name"], "Alice");
        assert_eq!(result["age"], 30);
        assert_eq!(result["city"], "Berlin");
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
        assert!(result.get("non_existent").is_none());
    }
}use serde_json::{Map, Value};
use std::env;
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::Path;

fn merge_json_files(file_paths: &[String], output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged_map = Map::new();

    for file_path in file_paths {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let json_value: Value = serde_json::from_reader(reader)?;

        if let Value::Object(map) = json_value {
            for (key, value) in map {
                merged_map.insert(key, value);
            }
        } else {
            return Err("Each JSON file must contain a JSON object at the top level".into());
        }
    }

    let output_file = File::create(output_path)?;
    serde_json::to_writer_pretty(output_file, &Value::Object(merged_map))?;

    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 4 {
        eprintln!("Usage: {} <output.json> <input1.json> <input2.json> ...", args[0]);
        std::process::exit(1);
    }

    let output_path = &args[1];
    let input_files = &args[2..];

    if let Err(e) = merge_json_files(input_files, output_path) {
        eprintln!("Error merging JSON files: {}", e);
        std::process::exit(1);
    }

    println!("Successfully merged {} JSON files into {}", input_files.len(), output_path);
}