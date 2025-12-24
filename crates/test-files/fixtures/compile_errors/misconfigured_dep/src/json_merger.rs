
use std::fs::{self, File};
use std::io::{BufReader, Write};
use serde_json::{Value, json};

pub fn merge_json_files(file_paths: &[String], output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged_array = Vec::new();

    for file_path in file_paths {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let json_value: Value = serde_json::from_reader(reader)?;

        if let Value::Array(arr) = json_value {
            merged_array.extend(arr);
        } else {
            merged_array.push(json_value);
        }
    }

    let output_file = File::create(output_path)?;
    serde_json::to_writer_pretty(output_file, &json!(merged_array))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_arrays() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();

        fs::write(file1.path(), r#"[{"id": 1}, {"id": 2}]"#).unwrap();
        fs::write(file2.path(), r#"[{"id": 3}, {"id": 4}]"#).unwrap();

        let paths = vec![
            file1.path().to_str().unwrap().to_string(),
            file2.path().to_str().unwrap().to_string(),
        ];

        merge_json_files(&paths, output_file.path().to_str().unwrap()).unwrap();

        let content = fs::read_to_string(output_file.path()).unwrap();
        let parsed: Value = serde_json::from_str(&content).unwrap();

        assert_eq!(parsed[0]["id"], 1);
        assert_eq!(parsed[3]["id"], 4);
        assert_eq!(parsed.as_array().unwrap().len(), 4);
    }

    #[test]
    fn test_merge_mixed_json() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();

        fs::write(file1.path(), r#"{"single": "object"}"#).unwrap();
        fs::write(file2.path(), r#"[{"array": "item"}]"#).unwrap();

        let paths = vec![
            file1.path().to_str().unwrap().to_string(),
            file2.path().to_str().unwrap().to_string(),
        ];

        merge_json_files(&paths, output_file.path().to_str().unwrap()).unwrap();

        let content = fs::read_to_string(output_file.path()).unwrap();
        let parsed: Value = serde_json::from_str(&content).unwrap();

        assert_eq!(parsed[0]["single"], "object");
        assert_eq!(parsed[1]["array"], "item");
        assert_eq!(parsed.as_array().unwrap().len(), 2);
    }
}