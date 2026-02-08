
use serde_json::{Value, Map};
use std::fs;
use std::path::Path;
use std::collections::HashSet;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P]) -> Result<Value, String> {
    if paths.is_empty() {
        return Err("No input files provided".to_string());
    }

    let mut merged = Map::new();
    let mut conflict_log = Vec::new();

    for (idx, path) in paths.iter().enumerate() {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read {}: {}", path.as_ref().display(), e))?;
        
        let json: Value = serde_json::from_str(&content)
            .map_err(|e| format!("Invalid JSON in {}: {}", path.as_ref().display(), e))?;

        if let Value::Object(obj) = json {
            merge_object(&mut merged, obj, idx, &mut conflict_log);
        } else {
            return Err("Top-level JSON must be an object".to_string());
        }
    }

    if !conflict_log.is_empty() {
        eprintln!("Conflicts detected during merge:");
        for conflict in &conflict_log {
            eprintln!("  - {}", conflict);
        }
    }

    Ok(Value::Object(merged))
}

fn merge_object(base: &mut Map<String, Value>, 
                incoming: Map<String, Value>, 
                file_index: usize,
                conflicts: &mut Vec<String>) {
    for (key, incoming_value) in incoming {
        match base.get_mut(&key) {
            Some(existing_value) => {
                handle_conflict(key, existing_value, incoming_value, file_index, conflicts);
            }
            None => {
                base.insert(key, incoming_value);
            }
        }
    }
}

fn handle_conflict(key: String, 
                   existing: &mut Value, 
                   incoming: Value,
                   file_index: usize,
                   conflicts: &mut Vec<String>) {
    match (existing, incoming) {
        (Value::Object(existing_obj), Value::Object(incoming_obj)) => {
            merge_object(existing_obj, incoming_obj, file_index, conflicts);
        }
        (Value::Array(existing_arr), Value::Array(incoming_arr)) => {
            merge_array(existing_arr, incoming_arr);
        }
        (existing_val, incoming_val) if existing_val == &incoming_val => {
            // Values are identical, no conflict
        }
        _ => {
            conflicts.push(format!("Key '{}' from file {} overwritten", key, file_index));
            *existing = incoming_val;
        }
    }
}

fn merge_array(dest: &mut Vec<Value>, src: Vec<Value>) {
    let mut seen = HashSet::new();
    
    // Add all existing unique values to set
    for item in dest.iter() {
        if let Ok(serialized) = serde_json::to_string(item) {
            seen.insert(serialized);
        }
    }
    
    // Add only new unique values from source
    for item in src {
        if let Ok(serialized) = serde_json::to_string(&item) {
            if !seen.contains(&serialized) {
                seen.insert(serialized.clone());
                dest.push(item);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tempfile::NamedTempFile;

    #[test]
    fn test_basic_merge() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        
        fs::write(&file1, r#"{"a": 1, "b": 2}"#).unwrap();
        fs::write(&file2, r#"{"c": 3, "d": 4}"#).unwrap();
        
        let result = merge_json_files(&[file1.path(), file2.path()]).unwrap();
        let expected = json!({
            "a": 1,
            "b": 2,
            "c": 3,
            "d": 4
        });
        
        assert_eq!(result, expected);
    }

    #[test]
    fn test_nested_merge() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        
        fs::write(&file1, r#"{"config": {"timeout": 30}}"#).unwrap();
        fs::write(&file2, r#"{"config": {"retries": 3}}"#).unwrap();
        
        let result = merge_json_files(&[file1.path(), file2.path()]).unwrap();
        let expected = json!({
            "config": {
                "timeout": 30,
                "retries": 3
            }
        });
        
        assert_eq!(result, expected);
    }
}