use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P], output_path: P) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged: HashMap<String, Value> = HashMap::new();

    for path in paths {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let data: Value = serde_json::from_reader(reader)?;

        if let Value::Object(map) = data {
            for (key, value) in map {
                merged.insert(key, value);
            }
        } else {
            return Err("Each JSON file must contain an object at the root".into());
        }
    }

    let output_file = File::create(output_path)?;
    let merged_value = Value::Object(
        merged
            .into_iter()
            .map(|(k, v)| (k, v))
            .collect()
    );
    serde_json::to_writer_pretty(output_file, &merged_value)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_files() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        let output = NamedTempFile::new().unwrap();

        let data1 = json!({"name": "Alice", "age": 30});
        let data2 = json!({"city": "Berlin", "active": true});

        serde_json::to_writer(&file1, &data1).unwrap();
        serde_json::to_writer(&file2, &data2).unwrap();

        let paths = [file1.path(), file2.path()];
        merge_json_files(&paths, output.path()).unwrap();

        let mut content = String::new();
        File::open(output.path())
            .unwrap()
            .read_to_string(&mut content)
            .unwrap();

        let parsed: Value = serde_json::from_str(&content).unwrap();
        assert_eq!(parsed["name"], "Alice");
        assert_eq!(parsed["age"], 30);
        assert_eq!(parsed["city"], "Berlin");
        assert_eq!(parsed["active"], true);
    }
}