use serde_json::{Map, Value};
use std::fs;
use std::io;
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(input_paths: &[P], output_path: P) -> io::Result<()> {
    let mut merged_map = Map::new();

    for input_path in input_paths {
        let content = fs::read_to_string(input_path)?;
        let json_value: Value = serde_json::from_str(&content)?;

        if let Value::Object(map) = json_value {
            for (key, value) in map {
                merged_map.insert(key, value);
            }
        } else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Input file does not contain a JSON object",
            ));
        }
    }

    let merged_value = Value::Object(merged_map);
    let serialized = serde_json::to_string_pretty(&merged_value)?;
    fs::write(output_path, serialized)?;

    Ok(())
}