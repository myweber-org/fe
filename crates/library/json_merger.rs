
use serde_json::{Map, Value};
use std::collections::HashSet;
use std::fs;
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P], output_path: P) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged = Map::new();
    let mut processed_keys = HashSet::new();
    let mut conflict_log = Vec::new();

    for path in paths {
        let content = fs::read_to_string(path)?;
        let json: Value = serde_json::from_str(&content)?;

        if let Value::Object(obj) = json {
            for (key, value) in obj {
                if processed_keys.contains(&key) {
                    conflict_log.push(format!("Conflict detected for key '{}'", key));
                    if let Some(existing) = merged.get_mut(&key) {
                        *existing = merge_values(existing.clone(), value);
                    }
                } else {
                    merged.insert(key.clone(), value);
                    processed_keys.insert(key);
                }
            }
        }
    }

    if !conflict_log.is_empty() {
        let log_path = output_path.as_ref().with_extension("conflicts.log");
        fs::write(log_path, conflict_log.join("\n"))?;
    }

    let output_json = Value::Object(merged);
    let formatted = serde_json::to_string_pretty(&output_json)?;
    fs::write(output_path, formatted)?;

    Ok(())
}

fn merge_values(v1: Value, v2: Value) -> Value {
    match (v1, v2) {
        (Value::Array(mut arr1), Value::Array(arr2)) => {
            arr1.extend(arr2);
            Value::Array(arr1)
        }
        (Value::Object(mut obj1), Value::Object(obj2)) => {
            for (k, v) in obj2 {
                if obj1.contains_key(&k) {
                    let existing = obj1.remove(&k).unwrap();
                    obj1.insert(k, merge_values(existing, v));
                } else {
                    obj1.insert(k, v);
                }
            }
            Value::Object(obj1)
        }
        (_, v2) => v2,
    }
}