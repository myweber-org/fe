
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read, Write};
use std::path::Path;

type JsonValue = serde_json::Value;

pub fn merge_json_files(file_paths: &[impl AsRef<Path>]) -> Result<JsonValue, Box<dyn std::error::Error>> {
    let mut merged_map = HashMap::new();

    for path in file_paths {
        let file = File::open(path.as_ref())?;
        let mut reader = BufReader::new(file);
        let mut contents = String::new();
        reader.read_to_string(&mut contents)?;

        let json_value: JsonValue = serde_json::from_str(&contents)?;

        if let JsonValue::Object(map) = json_value {
            for (key, value) in map {
                merged_map.insert(key, value);
            }
        } else {
            return Err("Each JSON file must contain a JSON object".into());
        }
    }

    Ok(JsonValue::Object(merged_map.into_iter().collect()))
}

pub fn write_merged_json(output_path: impl AsRef<Path>, json_value: &JsonValue) -> Result<(), Box<dyn std::error::Error>> {
    let json_string = serde_json::to_string_pretty(json_value)?;
    let mut file = File::create(output_path)?;
    file.write_all(json_string.as_bytes())?;
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

        writeln!(file1, r#"{"a": 1, "b": "test"}"#).unwrap();
        writeln!(file2, r#"{"c": true, "d": [1,2,3]}"#).unwrap();

        let paths = [file1.path(), file2.path()];
        let result = merge_json_files(&paths).unwrap();

        assert_eq!(result["a"], 1);
        assert_eq!(result["b"], "test");
        assert_eq!(result["c"], true);
        assert_eq!(result["d"], serde_json::json!([1, 2, 3]));
    }

    #[test]
    fn test_write_merged_json() {
        let json_value = serde_json::json!({
            "key1": "value1",
            "key2": 42
        });

        let output_file = NamedTempFile::new().unwrap();
        write_merged_json(output_file.path(), &json_value).unwrap();

        let mut content = String::new();
        File::open(output_file.path())
            .unwrap()
            .read_to_string(&mut content)
            .unwrap();

        let parsed: JsonValue = serde_json::from_str(&content).unwrap();
        assert_eq!(parsed, json_value);
    }
}