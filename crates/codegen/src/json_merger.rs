
use serde_json::{Value, Map};
use std::fs;
use std::env;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <output_file.json> <input1.json> [input2.json ...]", args[0]);
        process::exit(1);
    }

    let output_path = &args[1];
    let mut merged_map = Map::new();

    for input_path in args.iter().skip(2) {
        match fs::read_to_string(input_path) {
            Ok(content) => {
                match serde_json::from_str::<Value>(&content) {
                    Ok(Value::Object(map)) => {
                        for (key, value) in map {
                            merged_map.insert(key, value);
                        }
                    }
                    Ok(_) => eprintln!("Warning: {} does not contain a JSON object, skipping.", input_path),
                    Err(e) => eprintln!("Error parsing {}: {}", input_path, e),
                }
            }
            Err(e) => eprintln!("Error reading {}: {}", input_path, e),
        }
    }

    let merged_value = Value::Object(merged_map);
    match fs::write(output_path, serde_json::to_string_pretty(&merged_value).unwrap()) {
        Ok(_) => println!("Successfully merged JSON into {}", output_path),
        Err(e) => eprintln!("Error writing to {}: {}", output_path, e),
    }
}