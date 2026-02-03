
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: u8 = 0xAA;

fn xor_cipher(data: &mut [u8], key: u8) {
    for byte in data.iter_mut() {
        *byte ^= key;
    }
}

fn process_file(input_path: &str, output_path: &str, key: u8) -> io::Result<()> {
    let mut file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    xor_cipher(&mut buffer, key);

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&buffer)?;

    Ok(())
}

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() < 3 {
        eprintln!("Usage: {} <input_file> <output_file> [key]", args[0]);
        eprintln!("Key must be between 0-255 (default: {})", DEFAULT_KEY);
        std::process::exit(1);
    }

    let input_path = &args[1];
    let output_path = &args[2];
    
    let key = if args.len() > 3 {
        args[3].parse::<u8>().unwrap_or_else(|_| {
            eprintln!("Invalid key, using default: {}", DEFAULT_KEY);
            DEFAULT_KEY
        })
    } else {
        DEFAULT_KEY
    };

    if !Path::new(input_path).exists() {
        eprintln!("Error: Input file '{}' not found", input_path);
        std::process::exit(1);
    }

    match process_file(input_path, output_path, key) {
        Ok(_) => {
            println!("File processed successfully!");
            println!("Input:  {}", input_path);
            println!("Output: {}", output_path);
            println!("Key:    0x{:02X}", key);
        }
        Err(e) => {
            eprintln!("Error processing file: {}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}