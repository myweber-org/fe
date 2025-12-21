
use std::fs;
use std::io::{Read, Write};
use std::path::Path;
use rand::Rng;

const KEY_SIZE: usize = 32;

fn generate_key() -> Vec<u8> {
    let mut rng = rand::thread_rng();
    (0..KEY_SIZE).map(|_| rng.gen()).collect()
}

fn xor_operation(data: &[u8], key: &[u8]) -> Vec<u8> {
    data.iter()
        .zip(key.iter().cycle())
        .map(|(d, k)| d ^ k)
        .collect()
}

fn encrypt_file(input_path: &Path, output_path: &Path, key: &[u8]) -> std::io::Result<()> {
    let mut input_file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    input_file.read_to_end(&mut buffer)?;
    
    let encrypted_data = xor_operation(&buffer, key);
    
    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&encrypted_data)?;
    
    Ok(())
}

fn decrypt_file(input_path: &Path, output_path: &Path, key: &[u8]) -> std::io::Result<()> {
    encrypt_file(input_path, output_path, key)
}

fn save_key(key: &[u8], path: &Path) -> std::io::Result<()> {
    let mut key_file = fs::File::create(path)?;
    key_file.write_all(key)?;
    Ok(())
}

fn load_key(path: &Path) -> std::io::Result<Vec<u8>> {
    let mut key_file = fs::File::open(path)?;
    let mut key = Vec::new();
    key_file.read_to_end(&mut key)?;
    Ok(key)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() < 4 {
        eprintln!("Usage: {} <encrypt|decrypt> <input_file> <output_file> [key_file]", args[0]);
        std::process::exit(1);
    }
    
    let operation = &args[1];
    let input_path = Path::new(&args[2]);
    let output_path = Path::new(&args[3]);
    
    let key = if args.len() > 4 {
        let key_path = Path::new(&args[4]);
        match load_key(key_path) {
            Ok(k) => k,
            Err(e) => {
                eprintln!("Failed to load key: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        let new_key = generate_key();
        let key_path = Path::new("encryption_key.bin");
        if let Err(e) = save_key(&new_key, key_path) {
            eprintln!("Warning: Failed to save generated key: {}", e);
        }
        new_key
    };
    
    let result = match operation.as_str() {
        "encrypt" => encrypt_file(input_path, output_path, &key),
        "decrypt" => decrypt_file(input_path, output_path, &key),
        _ => {
            eprintln!("Invalid operation. Use 'encrypt' or 'decrypt'");
            std::process::exit(1);
        }
    };
    
    if let Err(e) = result {
        eprintln!("Operation failed: {}", e);
        std::process::exit(1);
    }
    
    println!("Operation completed successfully");
}