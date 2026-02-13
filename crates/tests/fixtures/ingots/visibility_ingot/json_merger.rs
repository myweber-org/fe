use serde_json::{Value, json};
use std::fs;
use std::path::Path;
use std::env;
use std::error::Error;

fn merge_json_files(file_paths: &[String]) -> Result<Value, Box<dyn Error>> {
    let mut merged_array = Vec::new();

    for file_path in file_paths {
        let content = fs::read_to_string(file_path)?;
        let json_value: Value = serde_json::from_str(&content)?;
        
        if let Value::Array(arr) = json_value {
            merged_array.extend(arr);
        } else {
            merged_array.push(json_value);
        }
    }

    Ok(json!(merged_array))
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().skip(1).collect();
    
    if args.is_empty() {
        eprintln!("Usage: json_merger <file1.json> <file2.json> ...");
        std::process::exit(1);
    }

    let merged = merge_json_files(&args)?;
    
    let output_path = Path::new("merged_output.json");
    fs::write(output_path, merged.to_string())?;
    
    println!("Successfully merged {} files into {}", args.len(), output_path.display());
    Ok(())
}