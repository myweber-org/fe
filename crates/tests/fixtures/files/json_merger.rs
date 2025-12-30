
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

        if let Value::Object(obj) = json_value {
            for (key, value) in obj {
                if value.is_object() {
                    if let Some(id_value) = value.get(dedup_key) {
                        let id_str = id_value.to_string();
                        if !seen_keys.contains(&id_str) {
                            seen_keys.insert(id_str);
                            merged_map.insert(key, value);
                        }
                    } else {
                        merged_map.insert(key, value);
                    }
                } else {
                    merged_map.insert(key, value);
                }
            }
        }
    }

    let output_file = File::create(output_path)?;
    let writer = BufWriter::new(output_file);
    serde_json::to_writer_pretty(writer, &Value::Object(merged_map))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_with_dedup() {
        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();

        let json1 = json!({
            "item1": {"id": "a", "name": "Alpha"},
            "item2": {"id": "b", "name": "Beta"}
        });

        let json2 = json!({
            "item3": {"id": "a", "name": "AlphaDuplicate"},
            "item4": {"id": "c", "name": "Gamma"}
        });

        write!(file1, "{}", json1).unwrap();
        write!(file2, "{}", json2).unwrap();

        merge_json_files(
            &[file1.path(), file2.path()],
            output_file.path(),
            "id",
        ).unwrap();

        let output_content = std::fs::read_to_string(output_file.path()).unwrap();
        let parsed: Value = serde_json::from_str(&output_content).unwrap();

        assert!(parsed.get("item1").is_some());
        assert!(parsed.get("item2").is_some());
        assert!(parsed.get("item3").is_none());
        assert!(parsed.get("item4").is_some());
    }
}use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read, Write};
use std::path::Path;

type JsonValue = serde_json::Value;

pub fn merge_json_files(file_paths: &[impl AsRef<Path>]) -> Result<JsonValue, Box<dyn std::error::Error>> {
    let mut merged_array = Vec::new();

    for path in file_paths {
        let file = File::open(path.as_ref())?;
        let mut reader = BufReader::new(file);
        let mut contents = String::new();
        reader.read_to_string(&mut contents)?;

        let json_value: JsonValue = serde_json::from_str(&contents)?;
        
        match json_value {
            JsonValue::Array(arr) => {
                merged_array.extend(arr);
            }
            _ => {
                merged_array.push(json_value);
            }
        }
    }

    Ok(JsonValue::Array(merged_array))
}

pub fn write_merged_json(output_path: impl AsRef<Path>, json_value: &JsonValue) -> Result<(), Box<dyn std::error::Error>> {
    let pretty_json = serde_json::to_string_pretty(json_value)?;
    let mut file = File::create(output_path)?;
    file.write_all(pretty_json.as_bytes())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_arrays() -> Result<(), Box<dyn std::error::Error>> {
        let json1 = r#"[{"id": 1, "name": "Alice"}, {"id": 2, "name": "Bob"}]"#;
        let json2 = r#"[{"id": 3, "name": "Charlie"}]"#;

        let mut file1 = NamedTempFile::new()?;
        let mut file2 = NamedTempFile::new()?;
        file1.write_all(json1.as_bytes())?;
        file2.write_all(json2.as_bytes())?;

        let paths = [file1.path(), file2.path()];
        let merged = merge_json_files(&paths)?;

        assert!(merged.is_array());
        let array = merged.as_array().unwrap();
        assert_eq!(array.len(), 3);
        assert_eq!(array[0]["name"], "Alice");
        assert_eq!(array[2]["name"], "Charlie");

        Ok(())
    }

    #[test]
    fn test_merge_single_json_object() -> Result<(), Box<dyn std::error::Error>> {
        let json = r#"{"status": "success", "code": 200}"#;
        let mut file = NamedTempFile::new()?;
        file.write_all(json.as_bytes())?;

        let merged = merge_json_files(&[file.path()])?;
        assert!(merged.is_array());
        let array = merged.as_array().unwrap();
        assert_eq!(array.len(), 1);
        assert_eq!(array[0]["code"], 200);

        Ok(())
    }
}