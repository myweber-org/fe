
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, Read, Write};
use std::path::Path;

use serde_json::{json, Value};

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P], output_path: P) -> Result<(), String> {
    let mut merged_array = Vec::new();

    for path in paths {
        let file = File::open(path).map_err(|e| format!("Failed to open {}: {}", path.as_ref().display(), e))?;
        let mut reader = BufReader::new(file);
        let mut content = String::new();
        reader.read_to_string(&mut content).map_err(|e| format!("Failed to read {}: {}", path.as_ref().display(), e))?;

        let json_value: Value = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse JSON from {}: {}", path.as_ref().display(), e))?;

        match json_value {
            Value::Array(arr) => merged_array.extend(arr),
            Value::Object(obj) => merged_array.push(Value::Object(obj)),
            _ => return Err(format!("Unsupported JSON structure in {}", path.as_ref().display())),
        }
    }

    let output_json = Value::Array(merged_array);
    let output_str = serde_json::to_string_pretty(&output_json)
        .map_err(|e| format!("Failed to serialize merged JSON: {}", e))?;

    fs::write(output_path, output_str)
        .map_err(|e| format!("Failed to write output file: {}", e))?;

    Ok(())
}

pub fn merge_json_with_conflict_resolution<P: AsRef<Path>>(paths: &[P], output_path: P, conflict_strategy: ConflictStrategy) -> Result<(), String> {
    let mut merged_map: HashMap<String, Value> = HashMap::new();

    for path in paths {
        let file = File::open(path).map_err(|e| format!("Failed to open {}: {}", path.as_ref().display(), e))?;
        let mut reader = BufReader::new(file);
        let mut content = String::new();
        reader.read_to_string(&mut content).map_err(|e| format!("Failed to read {}: {}", path.as_ref().display(), e))?;

        let json_value: Value = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse JSON from {}: {}", path.as_ref().display(), e))?;

        if let Value::Object(obj) = json_value {
            for (key, value) in obj {
                match conflict_strategy {
                    ConflictStrategy::Overwrite => {
                        merged_map.insert(key, value);
                    }
                    ConflictStrategy::KeepFirst => {
                        merged_map.entry(key).or_insert(value);
                    }
                    ConflictStrategy::MergeArrays => {
                        merged_map.entry(key)
                            .and_modify(|existing| {
                                if let (Value::Array(existing_arr), Value::Array(new_arr)) = (existing, &value) {
                                    let mut combined = existing_arr.clone();
                                    combined.extend(new_arr.clone());
                                    *existing = Value::Array(combined);
                                } else {
                                    *existing = value.clone();
                                }
                            })
                            .or_insert(value);
                    }
                }
            }
        } else {
            return Err(format!("Expected JSON object in {}", path.as_ref().display()));
        }
    }

    let output_json = Value::Object(merged_map.into_iter().collect());
    let output_str = serde_json::to_string_pretty(&output_json)
        .map_err(|e| format!("Failed to serialize merged JSON: {}", e))?;

    fs::write(output_path, output_str)
        .map_err(|e| format!("Failed to write output file: {}", e))?;

    Ok(())
}

#[derive(Debug, Clone, Copy)]
pub enum ConflictStrategy {
    Overwrite,
    KeepFirst,
    MergeArrays,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_files() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();

        fs::write(&file1, r#"[{"id": 1}, {"id": 2}]"#).unwrap();
        fs::write(&file2, r#"[{"id": 3}, {"id": 4}]"#).unwrap();

        merge_json_files(&[file1.path(), file2.path()], output_file.path()).unwrap();

        let content = fs::read_to_string(output_file.path()).unwrap();
        let parsed: Value = serde_json::from_str(&content).unwrap();
        assert!(parsed.is_array());
        assert_eq!(parsed.as_array().unwrap().len(), 4);
    }

    #[test]
    fn test_merge_with_conflict_resolution() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();

        fs::write(&file1, r#"{"a": 1, "b": [1, 2]}"#).unwrap();
        fs::write(&file2, r#"{"a": 2, "b": [3, 4], "c": 5}"#).unwrap();

        merge_json_with_conflict_resolution(
            &[file1.path(), file2.path()],
            output_file.path(),
            ConflictStrategy::MergeArrays
        ).unwrap();

        let content = fs::read_to_string(output_file.path()).unwrap();
        let parsed: Value = serde_json::from_str(&content).unwrap();
        let obj = parsed.as_object().unwrap();
        
        assert_eq!(obj.get("a").unwrap().as_i64().unwrap(), 2);
        assert_eq!(obj.get("b").unwrap().as_array().unwrap().len(), 4);
        assert_eq!(obj.get("c").unwrap().as_i64().unwrap(), 5);
    }
}