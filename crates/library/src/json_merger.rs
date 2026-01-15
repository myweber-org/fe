
use serde_json::{Value, Map};
use std::fs;
use std::path::Path;
use std::collections::HashSet;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P], output_path: P) -> Result<(), String> {
    if paths.is_empty() {
        return Err("No input files provided".to_string());
    }

    let mut merged: Map<String, Value> = Map::new();
    let mut conflict_log = Vec::new();

    for (idx, path) in paths.iter().enumerate() {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read {}: {}", path.as_ref().display(), e))?;
        
        let json: Value = serde_json::from_str(&content)
            .map_err(|e| format!("Invalid JSON in {}: {}", path.as_ref().display(), e))?;

        if let Value::Object(obj) = json {
            merge_object(&mut merged, obj, idx, &mut conflict_log);
        } else {
            return Err(format!("Root element in {} must be a JSON object", path.as_ref().display()));
        }
    }

    if !conflict_log.is_empty() {
        log_conflicts(&conflict_log);
    }

    let output_json = Value::Object(merged);
    let pretty_json = serde_json::to_string_pretty(&output_json)
        .map_err(|e| format!("Failed to serialize merged JSON: {}", e))?;

    fs::write(&output_path, pretty_json)
        .map_err(|e| format!("Failed to write output file: {}", e))?;

    Ok(())
}

fn merge_object(base: &mut Map<String, Value>, new: Map<String, Value>, file_index: usize, conflicts: &mut Vec<String>) {
    for (key, new_value) in new {
        match base.get_mut(&key) {
            Some(existing_value) => {
                if existing_value != &new_value {
                    if both_are_objects(existing_value, &new_value) {
                        if let (Value::Object(ref mut base_obj), Value::Object(new_obj)) = (existing_value, new_value) {
                            merge_object(base_obj, new_obj, file_index, conflicts);
                        }
                    } else {
                        conflicts.push(format!("Conflict for key '{}': existing {:?} vs new {:?} from file {}", 
                            key, existing_value, new_value, file_index));
                        *existing_value = new_value;
                    }
                }
            }
            None => {
                base.insert(key, new_value);
            }
        }
    }
}

fn both_are_objects(v1: &Value, v2: &Value) -> bool {
    matches!(v1, Value::Object(_)) && matches!(v2, Value::Object(_))
}

fn log_conflicts(conflicts: &[String]) {
    eprintln!("⚠️  {} conflicts detected during merge:", conflicts.len());
    for conflict in conflicts {
        eprintln!("  - {}", conflict);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use serde_json::json;

    #[test]
    fn test_basic_merge() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        let output = NamedTempFile::new().unwrap();

        fs::write(&file1, r#"{"a": 1, "b": 2}"#).unwrap();
        fs::write(&file2, r#"{"c": 3, "d": 4}"#).unwrap();

        merge_json_files(&[&file1, &file2], &output).unwrap();
        
        let content = fs::read_to_string(output).unwrap();
        let parsed: Value = serde_json::from_str(&content).unwrap();
        
        assert_eq!(parsed["a"], 1);
        assert_eq!(parsed["b"], 2);
        assert_eq!(parsed["c"], 3);
        assert_eq!(parsed["d"], 4);
    }

    #[test]
    fn test_nested_merge() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        let output = NamedTempFile::new().unwrap();

        fs::write(&file1, r#"{"common": {"x": 1}, "unique1": "value1"}"#).unwrap();
        fs::write(&file2, r#"{"common": {"y": 2}, "unique2": "value2"}"#).unwrap();

        merge_json_files(&[&file1, &file2], &output).unwrap();
        
        let content = fs::read_to_string(output).unwrap();
        let parsed: Value = serde_json::from_str(&content).unwrap();
        
        assert_eq!(parsed["common"]["x"], 1);
        assert_eq!(parsed["common"]["y"], 2);
        assert_eq!(parsed["unique1"], "value1");
        assert_eq!(parsed["unique2"], "value2");
    }
}