use serde_json::{Value, json};
use std::fs;
use std::path::Path;
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <output_file.json> <input1.json> [input2.json ...]", args[0]);
        std::process::exit(1);
    }

    let output_path = &args[1];
    let input_paths = &args[2..];

    let mut merged_array = Vec::new();

    for input_path in input_paths {
        let content = fs::read_to_string(input_path)?;
        let json_value: Value = serde_json::from_str(&content)?;
        merged_array.push(json_value);
    }

    let output_json = json!(merged_array);
    fs::write(output_path, output_json.to_string())?;

    println!("Successfully merged {} files into {}", input_paths.len(), output_path);
    Ok(())
}