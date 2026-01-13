
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
    let input_paths = &args[2..];
    let mut merged_map = Map::new();

    for (index, input_path) in input_paths.iter().enumerate() {
        let content = match fs::read_to_string(input_path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Failed to read {}: {}", input_path, e);
                process::exit(1);
            }
        };

        let json_value: Value = match serde_json::from_str(&content) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("Failed to parse JSON from {}: {}", input_path, e);
                process::exit(1);
            }
        };

        if let Value::Object(obj) = json_value {
            for (key, value) in obj {
                let final_key = if merged_map.contains_key(&key) {
                    format!("{}_{}", key, index)
                } else {
                    key
                };
                merged_map.insert(final_key, value);
            }
        } else {
            eprintln!("Top-level element in {} is not a JSON object", input_path);
            process::exit(1);
        }
    }

    let merged_value = Value::Object(merged_map);
    let json_string = match serde_json::to_string_pretty(&merged_value) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to serialize merged JSON: {}", e);
            process::exit(1);
        }
    };

    if let Err(e) = fs::write(output_path, json_string) {
        eprintln!("Failed to write to {}: {}", output_path, e);
        process::exit(1);
    }

    println!("Successfully merged {} files into {}", input_paths.len(), output_path);
}