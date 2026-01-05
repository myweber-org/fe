
use serde_json::{Map, Value};
use std::env;
use std::fs::File;
use std::io::{BufReader, Result};

fn merge_json_objects(files: Vec<String>) -> Result<Value> {
    let mut merged_map = Map::new();

    for file_path in files {
        let file = File::open(&file_path)?;
        let reader = BufReader::new(file);
        let json_data: Value = serde_json::from_reader(reader)?;

        if let Value::Object(map) = json_data {
            for (key, value) in map {
                merged_map.insert(key, value);
            }
        } else {
            eprintln!("Warning: {} does not contain a JSON object", file_path);
        }
    }

    Ok(Value::Object(merged_map))
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() {
        eprintln!("Usage: json_merger <file1.json> <file2.json> ...");
        std::process::exit(1);
    }

    let merged = merge_json_objects(args)?;
    println!("{}", serde_json::to_string_pretty(&merged)?);
    Ok(())
}