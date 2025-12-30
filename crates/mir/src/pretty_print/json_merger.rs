use serde_json::{Value, json};
use std::fs::{self, File};
use std::io::{BufReader, Write};
use std::path::Path;

pub fn merge_json_files(input_paths: &[&str], output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged_array = Vec::new();

    for input_path in input_paths {
        let path = Path::new(input_path);
        if !path.exists() {
            eprintln!("Warning: File {} not found, skipping.", input_path);
            continue;
        }

        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let json_value: Value = serde_json::from_reader(reader)?;

        match json_value {
            Value::Array(arr) => {
                merged_array.extend(arr);
            }
            _ => {
                merged_array.push(json_value);
            }
        }
    }

    let output_file = File::create(output_path)?;
    let merged_json = json!(merged_array);
    serde_json::to_writer_pretty(output_file, &merged_json)?;

    Ok(())
}

pub fn merge_json_directories(input_dirs: &[&str], output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut all_json_paths = Vec::new();

    for dir in input_dirs {
        let entries = fs::read_dir(dir)?;
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) == Some("json") {
                if let Some(path_str) = path.to_str() {
                    all_json_paths.push(path_str.to_string());
                }
            }
        }
    }

    let path_refs: Vec<&str> = all_json_paths.iter().map(|s| s.as_str()).collect();
    merge_json_files(&path_refs, output_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn test_merge_json_files() {
        let temp_dir = TempDir::new().unwrap();
        let file1_path = temp_dir.path().join("data1.json");
        let file2_path = temp_dir.path().join("data2.json");
        let output_path = temp_dir.path().join("merged.json");

        fs::write(&file1_path, r#"[{"id": 1}, {"id": 2}]"#).unwrap();
        fs::write(&file2_path, r#"{"id": 3}"#).unwrap();

        let inputs = [
            file1_path.to_str().unwrap(),
            file2_path.to_str().unwrap()
        ];

        merge_json_files(&inputs, output_path.to_str().unwrap()).unwrap();

        let content = fs::read_to_string(output_path).unwrap();
        let parsed: Value = serde_json::from_str(&content).unwrap();
        assert_eq!(parsed.as_array().unwrap().len(), 3);
    }
}