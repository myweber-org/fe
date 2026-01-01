
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