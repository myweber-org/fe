
use serde_json::{Value, Map};
use std::fs::{self, File};
use std::io::{self, BufReader, Write};
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(input_paths: &[P], output_path: P) -> io::Result<()> {
    let mut merged_map = Map::new();

    for path in input_paths {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let json_value: Value = serde_json::from_reader(reader)?;

        if let Value::Object(map) = json_value {
            for (key, value) in map {
                merged_map.insert(key, value);
            }
        } else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Input JSON is not an object",
            ));
        }
    }

    let merged_json = Value::Object(merged_map);
    let mut output_file = File::create(output_path)?;
    write!(output_file, "{}", merged_json.to_string())?;

    Ok(())
}

pub fn merge_json_directory<P: AsRef<Path>>(dir_path: P, output_path: P) -> io::Result<()> {
    let mut json_files = Vec::new();

    for entry in fs::read_dir(dir_path)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|ext| ext.to_str()) == Some("json") {
            json_files.push(path);
        }
    }

    if json_files.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "No JSON files found in directory",
        ));
    }

    merge_json_files(&json_files, output_path)
}