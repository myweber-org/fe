
use serde_json::{Value, Map};
use std::fs;
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P]) -> Result<Value, String> {
    let mut merged = Map::new();

    for path in paths {
        let content = fs::read_to_string(path).map_err(|e| format!("Failed to read {}: {}", path.as_ref().display(), e))?;
        let json: Value = serde_json::from_str(&content).map_err(|e| format!("Invalid JSON in {}: {}", path.as_ref().display(), e))?;

        if let Value::Object(obj) = json {
            merge_objects(&mut merged, obj);
        } else {
            return Err(format!("Top-level JSON must be an object in {}", path.as_ref().display()));
        }
    }

    Ok(Value::Object(merged))
}

fn merge_objects(target: &mut Map<String, Value>, source: Map<String, Value>) {
    for (key, source_value) in source {
        match target.get_mut(&key) {
            Some(target_value) => {
                if let (Value::Object(mut target_obj), Value::Object(source_obj)) = (target_value.clone(), source_value) {
                    merge_objects(&mut target_obj, source_obj);
                    target.insert(key, Value::Object(target_obj));
                } else if target_value != &source_value {
                    eprintln!("Conflict for key '{}': {:?} vs {:?}. Using source value.", key, target_value, source_value);
                    target.insert(key, source_value);
                }
            }
            None => {
                target.insert(key, source_value);
            }
        }
    }
}
use serde_json::{Map, Value};
use std::env;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <file1.json> [file2.json ...]", args[0]);
        std::process::exit(1);
    }

    let mut merged = Map::new();

    for file_path in &args[1..] {
        let path = Path::new(file_path);
        if !path.exists() {
            eprintln!("Warning: File '{}' not found, skipping.", file_path);
            continue;
        }

        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let json_value: Value = serde_json::from_reader(reader)?;

        if let Value::Object(map) = json_value {
            for (key, value) in map {
                merged.insert(key, value);
            }
        } else {
            eprintln!("Warning: '{}' does not contain a JSON object, skipping.", file_path);
        }
    }

    let output = Value::Object(merged);
    let stdout = std::io::stdout();
    let handle = stdout.lock();
    serde_json::to_writer_pretty(handle, &output)?;

    Ok(())
}