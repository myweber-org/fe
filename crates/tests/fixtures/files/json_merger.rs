
use serde_json::{Value, Map};
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(
    input_paths: &[P],
    output_path: P,
    dedup_key: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged_map = Map::new();
    let mut seen_keys = HashSet::new();

    for path in input_paths {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let json_value: Value = serde_json::from_reader(reader)?;

        if let Value::Object(obj) = json_value {
            for (key, value) in obj {
                if value.is_object() {
                    if let Some(id_value) = value.get(dedup_key) {
                        let id_str = id_value.to_string();
                        if !seen_keys.contains(&id_str) {
                            seen_keys.insert(id_str);
                            merged_map.insert(key, value);
                        }
                    } else {
                        merged_map.insert(key, value);
                    }
                } else {
                    merged_map.insert(key, value);
                }
            }
        }
    }

    let output_file = File::create(output_path)?;
    let writer = BufWriter::new(output_file);
    serde_json::to_writer_pretty(writer, &Value::Object(merged_map))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_with_dedup() {
        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();

        let json1 = json!({
            "item1": {"id": "a", "name": "Alpha"},
            "item2": {"id": "b", "name": "Beta"}
        });

        let json2 = json!({
            "item3": {"id": "a", "name": "AlphaDuplicate"},
            "item4": {"id": "c", "name": "Gamma"}
        });

        write!(file1, "{}", json1).unwrap();
        write!(file2, "{}", json2).unwrap();

        merge_json_files(
            &[file1.path(), file2.path()],
            output_file.path(),
            "id",
        ).unwrap();

        let output_content = std::fs::read_to_string(output_file.path()).unwrap();
        let parsed: Value = serde_json::from_str(&output_content).unwrap();

        assert!(parsed.get("item1").is_some());
        assert!(parsed.get("item2").is_some());
        assert!(parsed.get("item3").is_none());
        assert!(parsed.get("item4").is_some());
    }
}