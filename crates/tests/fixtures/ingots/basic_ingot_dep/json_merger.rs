
use serde_json::{Value, Map};
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;

pub fn merge_json_files(input_paths: &[impl AsRef<Path>], output_path: impl AsRef<Path>) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged_array = Vec::new();
    let mut seen_ids = HashSet::new();

    for path in input_paths {
        let file = File::open(path.as_ref())?;
        let reader = BufReader::new(file);
        let json_value: Value = serde_json::from_reader(reader)?;

        if let Value::Array(arr) = json_value {
            for item in arr {
                if let Some(obj) = item.as_object() {
                    if let Some(id_value) = obj.get("id") {
                        if let Some(id_str) = id_value.as_str() {
                            if !seen_ids.contains(id_str) {
                                seen_ids.insert(id_str.to_string());
                                merged_array.push(item.clone());
                            }
                        }
                    }
                }
            }
        }
    }

    let output_file = File::create(output_path.as_ref())?;
    let writer = BufWriter::new(output_file);
    serde_json::to_writer_pretty(writer, &merged_array)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_files() {
        let json1 = json!([
            {"id": "1", "name": "Alice"},
            {"id": "2", "name": "Bob"}
        ]);
        let json2 = json!([
            {"id": "2", "name": "Bob"},
            {"id": "3", "name": "Charlie"}
        ]);

        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();

        writeln!(file1, "{}", json1.to_string()).unwrap();
        writeln!(file2, "{}", json2.to_string()).unwrap();

        let inputs = [file1.path(), file2.path()];
        merge_json_files(&inputs, output_file.path()).unwrap();

        let output_content = std::fs::read_to_string(output_file.path()).unwrap();
        let parsed: Value = serde_json::from_str(&output_content).unwrap();

        assert_eq!(parsed.as_array().unwrap().len(), 3);
    }
}