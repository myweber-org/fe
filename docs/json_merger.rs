use serde_json::{Map, Value};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(input_paths: &[P], output_path: P) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged_map = Map::new();
    let mut key_counter: HashMap<String, usize> = HashMap::new();

    for input_path in input_paths {
        let file = File::open(input_path)?;
        let reader = BufReader::new(file);
        let json_data: Value = serde_json::from_reader(reader)?;

        if let Value::Object(obj) = json_data {
            for (key, value) in obj {
                let mut final_key = key.clone();
                while merged_map.contains_key(&final_key) {
                    let count = key_counter.entry(key.clone()).or_insert(0);
                    *count += 1;
                    final_key = format!("{}_{}", key, count);
                }
                merged_map.insert(final_key, value);
            }
        } else {
            return Err("Input JSON is not an object".into());
        }
    }

    let output_value = Value::Object(merged_map);
    let mut output_file = File::create(output_path)?;
    write!(output_file, "{}", output_value.to_string())?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_basic_objects() {
        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();

        write!(file1, r#"{"name": "Alice", "age": 30}"#).unwrap();
        write!(file2, r#"{"city": "London", "country": "UK"}"#).unwrap();

        let inputs = [file1.path(), file2.path()];
        merge_json_files(&inputs, output_file.path()).unwrap();

        let output_content = std::fs::read_to_string(output_file.path()).unwrap();
        let parsed: Value = serde_json::from_str(&output_content).unwrap();

        assert_eq!(parsed["name"], "Alice");
        assert_eq!(parsed["city"], "London");
    }

    #[test]
    fn test_merge_with_duplicate_keys() {
        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();

        write!(file1, r#"{"id": 1, "tag": "rust"}"#).unwrap();
        write!(file2, r#"{"id": 2, "tag": "cargo"}"#).unwrap();

        let inputs = [file1.path(), file2.path()];
        merge_json_files(&inputs, output_file.path()).unwrap();

        let output_content = std::fs::read_to_string(output_file.path()).unwrap();
        let parsed: Value = serde_json::from_str(&output_content).unwrap();

        assert_eq!(parsed["id"], 1);
        assert_eq!(parsed["id_1"], 2);
        assert_eq!(parsed["tag"], "rust");
        assert_eq!(parsed["tag_1"], "cargo");
    }
}
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub fn merge_json_files(file_paths: &[&str]) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let mut merged_map = HashMap::new();

    for path_str in file_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            return Err(format!("File not found: {}", path_str).into());
        }

        let content = fs::read_to_string(path)?;
        let json_value: serde_json::Value = serde_json::from_str(&content)?;

        if let serde_json::Value::Object(map) = json_value {
            for (key, value) in map {
                if merged_map.contains_key(&key) {
                    eprintln!("Warning: Duplicate key '{}' found in file '{}'. Overwriting.", key, path_str);
                }
                merged_map.insert(key, value);
            }
        } else {
            return Err("JSON root is not an object".into());
        }
    }

    Ok(serde_json::Value::Object(merged_map))
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

        let json1 = json!({
            "name": "Alice",
            "age": 30
        });
        let json2 = json!({
            "city": "Berlin",
            "country": "Germany"
        });

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

    #[test]
    fn test_duplicate_key_overwrites() {
        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();

        let json1 = json!({ "id": 100 });
        let json2 = json!({ "id": 200 });

        write!(file1, "{}", json1).unwrap();
        write!(file2, "{}", json2).unwrap();

        let paths = vec![
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ];

        let result = merge_json_files(&paths).unwrap();
        assert_eq!(result["id"], 200);
    }
}