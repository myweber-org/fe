use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, BufReader, Write};
use std::path::Path;

use serde_json::{Map, Value};

fn merge_json_objects(base: &mut Map<String, Value>, new: Map<String, Value>) {
    for (key, value) in new {
        if let Some(existing) = base.get_mut(&key) {
            if existing.is_object() && value.is_object() {
                if let (Some(existing_obj), Some(new_obj)) = (existing.as_object_mut(), value.as_object()) {
                    let mut new_map = Map::new();
                    for (k, v) in new_obj {
                        new_map.insert(k.clone(), v.clone());
                    }
                    merge_json_objects(existing_obj, new_map);
                }
            } else {
                base.insert(key, value);
            }
        } else {
            base.insert(key, value);
        }
    }
}

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P], output_path: P) -> io::Result<()> {
    let mut merged: Map<String, Value> = Map::new();

    for path in paths {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let json: Map<String, Value> = serde_json::from_reader(reader)?;
        merge_json_objects(&mut merged, json);
    }

    let output_file = File::create(output_path)?;
    serde_json::to_writer_pretty(output_file, &merged)?;
    Ok(())
}

pub fn merge_json_from_directory<P: AsRef<Path>>(dir_path: P, output_path: P) -> io::Result<()> {
    let mut json_paths = Vec::new();
    for entry in fs::read_dir(dir_path)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) == Some("json") {
            json_paths.push(path);
        }
    }

    if json_paths.is_empty() {
        return Err(io::Error::new(io::ErrorKind::NotFound, "No JSON files found in directory"));
    }

    merge_json_files(&json_paths, output_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_files() {
        let json1 = r#"{"name": "Alice", "age": 30, "address": {"city": "Paris"}}"#;
        let json2 = r#"{"name": "Bob", "age": 25, "address": {"country": "France"}}"#;

        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();
        file1.write_all(json1.as_bytes()).unwrap();
        file2.write_all(json2.as_bytes()).unwrap();

        let output_file = NamedTempFile::new().unwrap();
        let paths = [file1.path(), file2.path()];

        merge_json_files(&paths, output_file.path()).unwrap();

        let content = fs::read_to_string(output_file.path()).unwrap();
        let parsed: Value = serde_json::from_str(&content).unwrap();

        assert_eq!(parsed["name"], "Bob");
        assert_eq!(parsed["age"], 25);
        assert_eq!(parsed["address"]["city"], "Paris");
        assert_eq!(parsed["address"]["country"], "France");
    }
}