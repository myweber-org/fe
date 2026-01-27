use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

pub fn merge_json_files(file_paths: &[&str]) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let mut merged_map = HashMap::new();

    for path_str in file_paths {
        let path = Path::new(path_str);
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        let mut contents = String::new();
        reader.read_to_string(&mut contents)?;

        let json_value: serde_json::Value = serde_json::from_str(&contents)?;
        
        if let serde_json::Value::Object(map) = json_value {
            for (key, value) in map {
                merged_map.insert(key, value);
            }
        } else {
            return Err("Each JSON file must contain a JSON object".into());
        }
    }

    Ok(serde_json::Value::Object(merged_map))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_merge_json_files() {
        let dir = tempdir().unwrap();
        
        let file1_path = dir.path().join("data1.json");
        fs::write(&file1_path, r#"{"name": "test", "count": 42}"#).unwrap();
        
        let file2_path = dir.path().join("data2.json");
        fs::write(&file2_path, r#"{"enabled": true, "tags": ["rust", "json"]}"#).unwrap();

        let result = merge_json_files(&[
            file1_path.to_str().unwrap(),
            file2_path.to_str().unwrap(),
        ]).unwrap();

        assert_eq!(result["name"], "test");
        assert_eq!(result["count"], 42);
        assert_eq!(result["enabled"], true);
        assert!(result["tags"].is_array());
    }
}