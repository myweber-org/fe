use serde_json::{Value, json};
use std::fs;
use std::path::Path;
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <output_file.json> <input1.json> [input2.json ...]", args[0]);
        std::process::exit(1);
    }

    let output_path = &args[1];
    let input_paths = &args[2..];

    let mut merged_array = Vec::new();

    for input_path in input_paths {
        let content = fs::read_to_string(input_path)?;
        let json_value: Value = serde_json::from_str(&content)?;
        merged_array.push(json_value);
    }

    let output_json = json!(merged_array);
    fs::write(output_path, output_json.to_string())?;

    println!("Successfully merged {} files into {}", input_paths.len(), output_path);
    Ok(())
}
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

fn merge_objects(base: &mut Map<String, Value>, new: Map<String, Value>) {
    for (key, new_value) in new {
        match base.get_mut(&key) {
            Some(existing_value) => {
                if let (Value::Object(mut existing_obj), Value::Object(new_obj)) = (existing_value, new_value) {
                    merge_objects(&mut existing_obj, new_obj);
                } else {
                    *existing_value = new_value;
                }
            }
            None => {
                base.insert(key, new_value);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use serde_json::json;

    #[test]
    fn test_merge_json() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        let output = NamedTempFile::new().unwrap();

        let json1 = json!({
            "name": "test",
            "nested": {
                "value": 1
            }
        });

        let json2 = json!({
            "version": "1.0",
            "nested": {
                "extra": true
            }
        });

        fs::write(&file1, serde_json::to_string(&json1).unwrap()).unwrap();
        fs::write(&file2, serde_json::to_string(&json2).unwrap()).unwrap();

        merge_json_files(&[file1.path(), file2.path()], output.path()).unwrap();

        let result_content = fs::read_to_string(output.path()).unwrap();
        let result: Value = serde_json::from_str(&result_content).unwrap();

        assert_eq!(result["name"], "test");
        assert_eq!(result["version"], "1.0");
        assert_eq!(result["nested"]["value"], 1);
        assert_eq!(result["nested"]["extra"], true);
    }
}