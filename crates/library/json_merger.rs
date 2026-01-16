
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::Path;

pub fn merge_json_files(
    input_paths: &[impl AsRef<Path>],
    output_path: impl AsRef<Path>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged_map = Map::new();
    let mut key_counter: HashMap<String, usize> = HashMap::new();

    for input_path in input_paths {
        let file = File::open(input_path.as_ref())?;
        let reader = BufReader::new(file);
        let json_data: Value = serde_json::from_reader(reader)?;

        if let Value::Object(obj) = json_data {
            for (key, value) in obj {
                let mut final_key = key.clone();
                while merged_map.contains_key(&final_key) {
                    let count = key_counter.entry(key.clone()).or_insert(1);
                    *count += 1;
                    final_key = format!("{}_{}", key, count);
                }
                merged_map.insert(final_key, value);
            }
        } else {
            return Err("Input JSON is not an object".into());
        }
    }

    let output_file = File::create(output_path.as_ref())?;
    let merged_value = Value::Object(merged_map);
    serde_json::to_writer_pretty(output_file, &merged_value)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_files() {
        let json1 = r#"{"name": "Alice", "age": 30}"#;
        let json2 = r#"{"city": "London", "name": "Bob"}"#;

        let mut file1 = NamedTempFile::new().unwrap();
        file1.write_all(json1.as_bytes()).unwrap();
        let mut file2 = NamedTempFile::new().unwrap();
        file2.write_all(json2.as_bytes()).unwrap();

        let output_file = NamedTempFile::new().unwrap();

        merge_json_files(
            &[file1.path(), file2.path()],
            output_file.path(),
        ).unwrap();

        let mut content = String::new();
        File::open(output_file.path())
            .unwrap()
            .read_to_string(&mut content)
            .unwrap();

        let parsed: Value = serde_json::from_str(&content).unwrap();
        assert_eq!(parsed["name"], "Alice");
        assert_eq!(parsed["name_1"], "Bob");
        assert_eq!(parsed["age"], 30);
        assert_eq!(parsed["city"], "London");
    }
}
use serde_json::{Value, from_reader, json};
use std::fs::{self, File};
use std::io::{self, BufReader};
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P]) -> io::Result<Value> {
    let mut merged_array = Vec::new();

    for path in paths {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let json_value: Value = from_reader(reader)?;

        if let Value::Array(arr) = json_value {
            merged_array.extend(arr);
        } else {
            merged_array.push(json_value);
        }
    }

    Ok(json!(merged_array))
}

pub fn merge_json_directory<P: AsRef<Path>>(dir_path: P) -> io::Result<Value> {
    let mut json_paths = Vec::new();

    for entry in fs::read_dir(dir_path)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().map_or(false, |ext| ext == "json") {
            json_paths.push(path);
        }
    }

    merge_json_files(&json_paths)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_merge_json_files() {
        let dir = tempdir().unwrap();
        
        let file1_path = dir.path().join("data1.json");
        let file2_path = dir.path().join("data2.json");

        let mut file1 = File::create(&file1_path).unwrap();
        file1.write_all(b"[1, 2, 3]").unwrap();

        let mut file2 = File::create(&file2_path).unwrap();
        file2.write_all(b"[4, 5, 6]").unwrap();

        let result = merge_json_files(&[file1_path, file2_path]).unwrap();
        assert_eq!(result, json!([1, 2, 3, 4, 5, 6]));
    }

    #[test]
    fn test_merge_json_directory() {
        let dir = tempdir().unwrap();
        
        let file1_path = dir.path().join("a.json");
        let file2_path = dir.path().join("b.json");

        let mut file1 = File::create(&file1_path).unwrap();
        file1.write_all(b"{\"id\": 1}").unwrap();

        let mut file2 = File::create(&file2_path).unwrap();
        file2.write_all(b"{\"id\": 2}").unwrap();

        let result = merge_json_directory(dir.path()).unwrap();
        assert_eq!(result, json!([{"id": 1}, {"id": 2}]));
    }
}