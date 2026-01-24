
use serde_json::{Value, Map};
use std::fs;
use std::path::Path;
use std::collections::HashSet;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P], output_path: P) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged_map = Map::new();
    let mut processed_keys = HashSet::new();
    let mut conflict_log = Vec::new();

    for path in paths {
        let content = fs::read_to_string(path)?;
        let json_value: Value = serde_json::from_str(&content)?;

        if let Value::Object(map) = json_value {
            for (key, value) in map {
                if processed_keys.contains(&key) {
                    conflict_log.push(format!("Conflict detected for key '{}'", key));
                    if let Some(existing_value) = merged_map.get(&key) {
                        if existing_value.is_object() && value.is_object() {
                            if let (Value::Object(existing_obj), Value::Object(new_obj)) = (existing_value, &value) {
                                let mut combined_obj = existing_obj.clone();
                                for (sub_key, sub_value) in new_obj {
                                    combined_obj.insert(sub_key.clone(), sub_value.clone());
                                }
                                merged_map.insert(key, Value::Object(combined_obj));
                            }
                        } else if existing_value.is_array() && value.is_array() {
                            if let (Value::Array(existing_arr), Value::Array(new_arr)) = (existing_value, &value) {
                                let mut combined_arr = existing_arr.clone();
                                combined_arr.extend(new_arr.clone());
                                merged_map.insert(key, Value::Array(combined_arr));
                            }
                        } else {
                            conflict_log.push(format!("Overwriting key '{}' with new value", key));
                            merged_map.insert(key, value);
                        }
                    }
                } else {
                    merged_map.insert(key.clone(), value);
                    processed_keys.insert(key);
                }
            }
        }
    }

    let output_value = Value::Object(merged_map);
    let output_json = serde_json::to_string_pretty(&output_value)?;
    fs::write(output_path, output_json)?;

    if !conflict_log.is_empty() {
        let log_content = conflict_log.join("\n");
        fs::write("merge_conflicts.log", log_content)?;
        println!("Merged with conflicts. See merge_conflicts.log for details.");
    } else {
        println!("Merge completed successfully without conflicts.");
    }

    Ok(())
}