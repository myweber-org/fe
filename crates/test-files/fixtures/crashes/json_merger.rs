use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, BufReader};
use std::path::Path;

use serde_json::{Map, Value};

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P]) -> io::Result<Value> {
    let mut merged_map = Map::new();

    for path in paths {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let json_value: Value = serde_json::from_reader(reader)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        if let Value::Object(map) = json_value {
            for (key, value) in map {
                merged_map.insert(key, value);
            }
        } else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "JSON root must be an object",
            ));
        }
    }

    Ok(Value::Object(merged_map))
}

pub fn merge_json_directories<P: AsRef<Path>>(dir_paths: &[P]) -> io::Result<HashMap<String, Value>> {
    let mut result = HashMap::new();

    for dir_path in dir_paths {
        let entries = fs::read_dir(dir_path)?;
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                let file_stem = path.file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown")
                    .to_string();

                let file = File::open(&path)?;
                let reader = BufReader::new(file);
                let json_value: Value = serde_json::from_reader(reader)
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

                result.insert(file_stem, json_value);
            }
        }
    }

    Ok(result)
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
        assert_eq!(result["d"][0], 1);
    }

    #[test]
    fn test_merge_json_directories() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("config.json");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, r#"{"port": 8080}"#).unwrap();

        let dirs = [dir.path()];
        let result = merge_json_directories(&dirs).unwrap();

        assert_eq!(result["config"]["port"], 8080);
    }
}