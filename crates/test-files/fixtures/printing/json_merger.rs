use serde_json::{Value, from_reader, to_writer_pretty};
use std::fs::{File, read_dir};
use std::io::{self, BufReader};
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(dir_path: P, output_file: P) -> io::Result<()> {
    let mut merged_array = Vec::new();

    for entry in read_dir(dir_path)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("json") {
            let file = File::open(&path)?;
            let reader = BufReader::new(file);
            let json_value: Value = from_reader(reader).map_err(|e| {
                io::Error::new(io::ErrorKind::InvalidData, format!("Failed to parse JSON from {:?}: {}", path, e))
            })?;
            merged_array.push(json_value);
        }
    }

    let output = File::create(output_file)?;
    to_writer_pretty(output, &merged_array).map_err(|e| {
        io::Error::new(io::ErrorKind::Other, format!("Failed to write merged JSON: {}", e))
    })
}