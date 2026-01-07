
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read, Write};
use std::path::Path;

type JsonValue = serde_json::Value;
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub fn merge_json_files(file_paths: &[impl AsRef<Path>]) -> Result<JsonValue> {
    let mut merged = HashMap::new();

    for path in file_paths {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        let mut contents = String::new();
        reader.read_to_string(&mut contents)?;

        let json_data: JsonValue = serde_json::from_str(&contents)?;
        
        if let JsonValue::Object(map) = json_data {
            for (key, value) in map {
                merged.insert(key, value);
            }
        } else {
            return Err("Each JSON file must contain an object".into());
        }
    }

    Ok(serde_json::to_value(merged)?)
}

pub fn write_merged_json(output_path: impl AsRef<Path>, data: &JsonValue) -> Result<()> {
    let json_string = serde_json::to_string_pretty(data)?;
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
    fn test_merge_json_files() -> Result<()> {
        let json1 = r#"{"name": "test", "value": 42}"#;
        let json2 = r#"{"enabled": true, "tags": ["rust", "json"]}"#;

        let mut file1 = NamedTempFile::new()?;
        let mut file2 = NamedTempFile::new()?;
        
        file1.write_all(json1.as_bytes())?;
        file2.write_all(json2.as_bytes())?;

        let merged = merge_json_files(&[file1.path(), file2.path()])?;
        
        assert!(merged.get("name").is_some());
        assert!(merged.get("enabled").is_some());
        assert_eq!(merged["value"], 42);
        assert_eq!(merged["enabled"], true);
        
        Ok(())
    }
}