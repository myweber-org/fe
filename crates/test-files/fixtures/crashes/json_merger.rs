use serde_json::{Value, json};
use std::fs::{self, File};
use std::io::{BufReader, Result};
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P], output_path: P) -> Result<()> {
    let mut merged_array = Vec::new();

    for path in paths {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let json_value: Value = serde_json::from_reader(reader)?;
        merged_array.push(json_value);
    }

    let output_file = File::create(output_path)?;
    serde_json::to_writer_pretty(output_file, &json!(merged_array))?;
    Ok(())
}

pub fn merge_json_files_in_directory<P: AsRef<Path>>(dir_path: P, output_path: P) -> Result<()> {
    let mut json_paths = Vec::new();

    for entry in fs::read_dir(dir_path)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) == Some("json") {
            json_paths.push(path);
        }
    }

    merge_json_files(&json_paths, output_path)
}