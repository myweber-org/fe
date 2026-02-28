
use serde_json::{Value, Map};
use std::fs::{self, File};
use std::io::{self, BufReader, Write};
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(input_paths: &[P], output_path: P) -> io::Result<()> {
    let mut merged_map = Map::new();

    for path in input_paths {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let json_value: Value = serde_json::from_reader(reader)?;

        if let Value::Object(map) = json_value {
            for (key, value) in map {
                merged_map.insert(key, value);
            }
        } else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Input JSON is not an object",
            ));
        }
    }

    let merged_json = Value::Object(merged_map);
    let mut output_file = File::create(output_path)?;
    write!(output_file, "{}", merged_json.to_string())?;

    Ok(())
}

pub fn merge_json_directory<P: AsRef<Path>>(dir_path: P, output_path: P) -> io::Result<()> {
    let mut json_files = Vec::new();

    for entry in fs::read_dir(dir_path)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|ext| ext.to_str()) == Some("json") {
            json_files.push(path);
        }
    }

    if json_files.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "No JSON files found in directory",
        ));
    }

    merge_json_files(&json_files, output_path)
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

        if let Some(obj) = json_value.as_object() {
            for (key, value) in obj {
                merged_map.insert(key.clone(), value.clone());
            }
        }
    }

    Ok(serde_json::Value::Object(
        merged_map.into_iter().collect()
    ))
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
        writeln!(file2, r#"{"city": "Berlin", "age": 31}"#).unwrap();

        let result = merge_json_files(&[
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap()
        ]).unwrap();

        assert_eq!(result["name"], "Alice");
        assert_eq!(result["city"], "Berlin");
        assert_eq!(result["age"], 31);
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

        if let Some(obj) = json_value.as_object() {
            for (key, value) in obj {
                merged_map.insert(key.clone(), value.clone());
            }
        }
    }

    Ok(serde_json::Value::Object(
        merged_map.into_iter().collect()
    ))
}