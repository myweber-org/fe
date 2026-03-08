use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub fn merge_json_files(file_paths: &[&str]) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let mut merged = HashMap::new();

    for path_str in file_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            eprintln!("Warning: File {} not found, skipping.", path_str);
            continue;
        }

        let content = fs::read_to_string(path)?;
        let json_value: serde_json::Value = serde_json::from_str(&content)?;

        if let serde_json::Value::Object(map) = json_value {
            for (key, value) in map {
                merged.insert(key, value);
            }
        } else {
            eprintln!("Warning: {} does not contain a JSON object, skipping.", path_str);
        }
    }

    Ok(serde_json::Value::Object(merged.into_iter().collect()))
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

        let json1 = json!({"name": "Alice", "age": 30});
        let json2 = json!({"city": "Berlin", "country": "Germany"});

        write!(file1, "{}", json1).unwrap();
        write!(file2, "{}", json2).unwrap();

        let paths = vec![
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ];

        let result = merge_json_files(&paths).unwrap();
        let expected = json!({
            "name": "Alice",
            "age": 30,
            "city": "Berlin",
            "country": "Germany"
        });

        assert_eq!(result, expected);
    }
}use serde_json::{json, Value};
use std::fs;
use std::path::Path;

pub fn merge_json_files(file_paths: &[&str], output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged_array = Vec::new();

    for file_path in file_paths {
        let content = fs::read_to_string(file_path)?;
        let json_value: Value = serde_json::from_str(&content)?;
        merged_array.push(json_value);
    }

    let output_json = json!(merged_array);
    let output_str = serde_json::to_string_pretty(&output_json)?;

    let output_dir = Path::new(output_path).parent().unwrap();
    if !output_dir.exists() {
        fs::create_dir_all(output_dir)?;
    }

    fs::write(output_path, output_str)?;
    Ok(())
}use serde_json::{Value, from_reader, to_writer_pretty};
use std::fs::{File, read_dir};
use std::io::{self, BufReader};
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(dir_path: P, output_path: P) -> io::Result<()> {
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

    let output_file = File::create(output_path)?;
    to_writer_pretty(output_file, &merged_array).map_err(|e| {
        io::Error::new(io::ErrorKind::Other, format!("Failed to write merged JSON: {}", e))
    })?;

    Ok(())
}