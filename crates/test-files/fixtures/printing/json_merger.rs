use serde_json::{Value, from_reader, to_writer_pretty};
use std::fs::{File, read_dir};
use std::io::{self, BufReader};
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(dir_path: P, output_file: P) -> io::Result<()> {
    let mut merged_array = Vec::new();

    for entry in read_dir(dir_path)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("json") {
            let file = File::open(&path)?;
            let reader = BufReader::new(file);
            let json_value: Value = from_reader(reader).map_err(|e| {
                io::Error::new(io::ErrorKind::InvalidData, format!("Failed to parse JSON from {:?}: {}", path, e))
            })?;
            merged_array.push(json_value);
        }
    }

    let output = File::create(output_file)?;
    to_writer_pretty(output, &merged_array).map_err(|e| {
        io::Error::new(io::ErrorKind::Other, format!("Failed to write merged JSON: {}", e))
    })
}
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
        writeln!(file2, r#"{"city": "Berlin", "active": true}"#).unwrap();

        let result = merge_json_files(&[
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ]).unwrap();

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
        ]).unwrap();

        assert_eq!(result["data"], "test");
        assert!(result.get("non_existent").is_none());
    }
}