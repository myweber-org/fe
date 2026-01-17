
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

        if let Value::Array(items) = json_value {
            for item in items {
                if let Value::Object(obj) = item {
                    if let Some(key_value) = obj.get(dedup_key) {
                        let key_string = key_value.to_string();
                        if !seen_keys.contains(&key_string) {
                            seen_keys.insert(key_string.clone());
                            merged_map.insert(key_string, Value::Object(obj));
                        }
                    }
                }
            }
        }
    }

    let output_array: Vec<Value> = merged_map.into_values().collect();
    let output_file = File::create(output_path)?;
    let writer = BufWriter::new(output_file);
    serde_json::to_writer_pretty(writer, &output_array)?;

    Ok(())
}