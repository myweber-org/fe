
use serde_json::{Map, Value};
use std::env;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

fn merge_json_files(file_paths: &[String]) -> Result<Value, Box<dyn std::error::Error>> {
    let mut merged_map = Map::new();

    for path_str in file_paths {
        let path = Path::new(path_str);
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let json_value: Value = serde_json::from_reader(reader)?;

        if let Value::Object(map) = json_value {
            for (key, value) in map {
                merged_map.insert(key, value);
            }
        } else {
            return Err("Input file does not contain a JSON object".into());
        }
    }

    Ok(Value::Object(merged_map))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().skip(1).collect();
    
    if args.is_empty() {
        eprintln!("Usage: json_merger <file1.json> <file2.json> ...");
        std::process::exit(1);
    }

    let merged = merge_json_files(&args)?;
    println!("{}", serde_json::to_string_pretty(&merged)?);
    Ok(())
}