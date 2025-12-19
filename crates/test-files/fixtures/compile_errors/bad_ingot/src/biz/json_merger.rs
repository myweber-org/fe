use serde_json::{json, Value};
use std::fs;
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P], output_path: P) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged_array = Vec::new();

    for path in paths {
        let content = fs::read_to_string(path)?;
        let parsed: Value = serde_json::from_str(&content)?;
        merged_array.push(parsed);
    }

    let output_value = Value::Array(merged_array);
    let output_json = serde_json::to_string_pretty(&output_value)?;
    fs::write(output_path, output_json)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_files() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();

        fs::write(&file1, r#"{"id": 1, "name": "test1"}"#).unwrap();
        fs::write(&file2, r#"{"id": 2, "name": "test2"}"#).unwrap();

        let paths = [file1.path(), file2.path()];
        merge_json_files(&paths, output_file.path()).unwrap();

        let output_content = fs::read_to_string(output_file.path()).unwrap();
        let expected = r#"[
  {
    "id": 1,
    "name": "test1"
  },
  {
    "id": 2,
    "name": "test2"
  }
]"#;
        assert_eq!(output_content.trim(), expected.trim());
    }
}