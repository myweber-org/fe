
use serde_json::{Value, Map};
use std::fs;
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P], output_path: P) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged = Map::new();
    
    for path in paths {
        let content = fs::read_to_string(path)?;
        let json: Value = serde_json::from_str(&content)?;
        
        if let Value::Object(obj) = json {
            merge_objects(&mut merged, obj);
        }
    }
    
    let output_json = Value::Object(merged);
    let serialized = serde_json::to_string_pretty(&output_json)?;
    fs::write(output_path, serialized)?;
    
    Ok(())
}

fn merge_objects(target: &mut Map<String, Value>, source: Map<String, Value>) {
    for (key, source_value) in source {
        match target.get_mut(&key) {
            Some(target_value) => {
                if let (Value::Object(mut target_obj), Value::Object(source_obj)) = (target_value.clone(), source_value.clone()) {
                    merge_objects(&mut target_obj, source_obj);
                    target.insert(key, Value::Object(target_obj));
                } else if target_value != &source_value {
                    let merged_array = Value::Array(vec![target_value.clone(), source_value]);
                    target.insert(key, merged_array);
                }
            }
            None => {
                target.insert(key, source_value);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_basic_merge() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        let output = NamedTempFile::new().unwrap();
        
        fs::write(&file1, r#"{"a": 1, "b": {"x": 10}}"#).unwrap();
        fs::write(&file2, r#"{"c": 3, "b": {"y": 20}}"#).unwrap();
        
        merge_json_files(&[&file1, &file2], &output).unwrap();
        
        let result = fs::read_to_string(output).unwrap();
        assert!(result.contains("\"a\": 1"));
        assert!(result.contains("\"c\": 3"));
        assert!(result.contains("\"x\": 10"));
        assert!(result.contains("\"y\": 20"));
    }
    
    #[test]
    fn test_conflict_resolution() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        let output = NamedTempFile::new().unwrap();
        
        fs::write(&file1, r#"{"version": "1.0"}"#).unwrap();
        fs::write(&file2, r#"{"version": "2.0"}"#).unwrap();
        
        merge_json_files(&[&file1, &file2], &output).unwrap();
        
        let result = fs::read_to_string(output).unwrap();
        assert!(result.contains(r#""version": ["1.0", "2.0"]"#));
    }
}