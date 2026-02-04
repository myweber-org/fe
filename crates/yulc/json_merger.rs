
use serde_json::{Value, Map};
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(
    input_paths: &[P],
    output_path: P,
    dedup_key: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged_map = Map::new();
    let mut seen_keys = HashSet::new();

    for path in input_paths {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let json_value: Value = serde_json::from_reader(reader)?;

        if let Value::Array(items) = json_value {
            for item in items {
                if let Value::Object(obj) = item {
                    if let Some(key_value) = obj.get(dedup_key) {
                        let key_string = key_value.to_string();
                        if !seen_keys.contains(&key_string) {
                            seen_keys.insert(key_string.clone());
                            merged_map.insert(key_string, Value::Object(obj));
                        }
                    }
                }
            }
        }
    }

    let output_array: Vec<Value> = merged_map.into_values().collect();
    let output_file = File::create(output_path)?;
    let writer = BufWriter::new(output_file);
    serde_json::to_writer_pretty(writer, &output_array)?;

    Ok(())
}
use std::collections::HashMap;
use std::fs;
use std::path::Path;

type JsonMap = HashMap<String, serde_json::Value>;

pub fn merge_json_files(file_paths: &[&str]) -> Result<JsonMap, Box<dyn std::error::Error>> {
    let mut merged = JsonMap::new();

    for path_str in file_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            continue;
        }

        let content = fs::read_to_string(path)?;
        let json_value: serde_json::Value = serde_json::from_str(&content)?;

        if let serde_json::Value::Object(obj) = json_value {
            for (key, value) in obj {
                merged.insert(key, value);
            }
        }
    }

    Ok(merged)
}

pub fn write_merged_json(output_path: &str, data: &JsonMap) -> Result<(), Box<dyn std::error::Error>> {
    let json_string = serde_json::to_string_pretty(data)?;
    fs::write(output_path, json_string)?;
    Ok(())
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

        writeln!(file1, r#"{"name": "test", "count": 42}"#).unwrap();
        writeln!(file2, r#"{"enabled": true, "tags": ["rust", "json"]}"#).unwrap();

        let paths = vec![
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ];

        let result = merge_json_files(&paths).unwrap();
        assert_eq!(result.len(), 4);
        assert_eq!(result.get("name").unwrap(), "test");
        assert_eq!(result.get("count").unwrap(), 42);
        assert_eq!(result.get("enabled").unwrap(), true);
    }
}