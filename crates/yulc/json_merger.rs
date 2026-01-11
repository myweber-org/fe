use serde_json::{Map, Value};
use std::fs;
use std::path::Path;

pub fn merge_json_files(file_paths: &[&str], output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged_object = Map::new();

    for file_path in file_paths {
        let content = fs::read_to_string(file_path)?;
        let json_value: Value = serde_json::from_str(&content)?;

        if let Value::Object(obj) = json_value {
            for (key, value) in obj {
                merged_object.insert(key, value);
            }
        } else {
            return Err("Each JSON file must contain a JSON object".into());
        }
    }

    let output_json = Value::Object(merged_object);
    let serialized = serde_json::to_string_pretty(&output_json)?;

    let output_dir = Path::new(output_path).parent().unwrap();
    if !output_dir.exists() {
        fs::create_dir_all(output_dir)?;
    }

    fs::write(output_path, serialized)?;
    Ok(())
}