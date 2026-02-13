
use serde_json::{Map, Value};
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P], output_path: P) -> Result<(), String> {
    if paths.is_empty() {
        return Err("No input files provided".to_string());
    }

    let mut merged = Map::new();
    let mut key_sources = Map::new();

    for (idx, path) in paths.iter().enumerate() {
        let file = File::open(path).map_err(|e| format!("Failed to open {}: {}", path.as_ref().display(), e))?;
        let reader = BufReader::new(file);
        let json: Map<String, Value> = serde_json::from_reader(reader)
            .map_err(|e| format!("Failed to parse {}: {}", path.as_ref().display(), e))?;

        for (key, value) in json {
            if let Some(existing) = merged.get(&key) {
                if existing != &value {
                    let source_info = key_sources.entry(key.clone()).or_insert_with(Vec::new);
                    source_info.push(format!("File {}: {:?}", idx + 1, existing));
                    source_info.push(format!("File {}: {:?}", idx + 1, value));
                    merged.insert(format!("{}_conflict", key), Value::Array(source_info.clone().into_iter().map(Value::String).collect()));
                }
            } else {
                merged.insert(key.clone(), value);
                key_sources.insert(key, vec![format!("File {}", idx + 1)]);
            }
        }
    }

    let output_file = File::create(&output_path)
        .map_err(|e| format!("Failed to create output file: {}", e))?;
    serde_json::to_writer_pretty(output_file, &Value::Object(merged))
        .map_err(|e| format!("Failed to write merged JSON: {}", e))?;

    Ok(())
}

pub fn find_unique_keys<P: AsRef<Path>>(paths: &[P]) -> Result<Vec<String>, String> {
    let mut all_keys = HashSet::new();
    let mut common_keys = HashSet::new();
    let mut first = true;

    for path in paths {
        let file = File::open(path).map_err(|e| format!("Failed to open {}: {}", path.as_ref().display(), e))?;
        let reader = BufReader::new(file);
        let json: Map<String, Value> = serde_json::from_reader(reader)
            .map_err(|e| format!("Failed to parse {}: {}", path.as_ref().display(), e))?;

        let current_keys: HashSet<_> = json.keys().cloned().collect();
        all_keys.extend(current_keys.iter().cloned());

        if first {
            common_keys = current_keys;
            first = false;
        } else {
            common_keys = common_keys.intersection(&current_keys).cloned().collect();
        }
    }

    let unique_keys: Vec<String> = all_keys.difference(&common_keys).cloned().collect();
    Ok(unique_keys)
}