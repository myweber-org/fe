
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Write};
use serde_json::{Value, Map};

fn merge_json_files(file_paths: &[&str], output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged_map: Map<String, Value> = Map::new();

    for path in file_paths {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let json_data: Value = serde_json::from_reader(reader)?;

        if let Value::Object(map) = json_data {
            for (key, value) in map {
                merged_map.insert(key, value);
            }
        } else {
            return Err("Each JSON file must contain a JSON object".into());
        }
    }

    let output_file = File::create(output_path)?;
    serde_json::to_writer_pretty(output_file, &Value::Object(merged_map))?;
    Ok(())
}

fn main() {
    let files_to_merge = vec!["data1.json", "data2.json", "data3.json"];
    let output_file = "merged_output.json";

    match merge_json_files(&files_to_merge, output_file) {
        Ok(()) => println!("Successfully merged JSON files into {}", output_file),
        Err(e) => eprintln!("Error merging JSON files: {}", e),
    }
}
use serde_json::{Value, Map};
use std::fs;
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P]) -> Result<Value, Box<dyn std::error::Error>> {
    let mut merged = Map::new();

    for path in paths {
        let content = fs::read_to_string(path)?;
        let json: Value = serde_json::from_str(&content)?;

        if let Value::Object(obj) = json {
            for (key, value) in obj {
                merge_value(&mut merged, key, value);
            }
        }
    }

    Ok(Value::Object(merged))
}

fn merge_value(map: &mut Map<String, Value>, key: String, new_value: Value) {
    match map.get_mut(&key) {
        Some(existing) => {
            if let (Value::Object(existing_obj), Value::Object(new_obj)) = (existing, &new_value) {
                let mut existing_obj = existing_obj.clone();
                for (k, v) in new_obj {
                    merge_value(&mut existing_obj, k.clone(), v.clone());
                }
                *existing = Value::Object(existing_obj);
            } else if let (Value::Array(existing_arr), Value::Array(new_arr)) = (existing, &new_value) {
                let mut combined = existing_arr.clone();
                combined.extend(new_arr.clone());
                *existing = Value::Array(combined);
            } else {
                *existing = new_value;
            }
        }
        None => {
            map.insert(key, new_value);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json() -> Result<(), Box<dyn std::error::Error>> {
        let file1 = NamedTempFile::new()?;
        let file2 = NamedTempFile::new()?;

        fs::write(&file1, r#"{"a": 1, "b": {"x": 10}}"#)?;
        fs::write(&file2, r#"{"a": 2, "b": {"y": 20}, "c": [1,2]}"#)?;

        let result = merge_json_files(&[file1.path(), file2.path()])?;
        
        assert_eq!(result["a"], 2);
        assert_eq!(result["b"]["x"], 10);
        assert_eq!(result["b"]["y"], 20);
        assert_eq!(result["c"], json!([1, 2]));

        Ok(())
    }
}