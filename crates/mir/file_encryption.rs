
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: u8 = 0x55;

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
    
    if args.len() != 3 {
        eprintln!("Usage: {} <input_file> <output_file>", args[0]);
        std::process::exit(1);
    }
    
    let input_path = &args[1];
    let output_path = &args[2];
    
    if !Path::new(input_path).exists() {
        eprintln!("Error: Input file '{}' does not exist", input_path);
        std::process::exit(1);
    }
    
    match process_file(input_path, output_path, DEFAULT_KEY) {
        Ok(_) => println!("File processed successfully"),
        Err(e) => eprintln!("Error processing file: {}", e),
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    
    #[test]
    fn test_xor_cipher() {
        let mut data = vec![0x00, 0xFF, 0x55, 0xAA];
        let original = data.clone();
        let key = 0x55;
        
        xor_cipher(&mut data, key);
        assert_ne!(data, original);
        
        xor_cipher(&mut data, key);
        assert_eq!(data, original);
    }
    
    #[test]
    fn test_file_encryption() {
        let test_data = b"Hello, World!";
        let input_path = "test_input.txt";
        let output_path = "test_output.txt";
        
        fs::write(input_path, test_data).unwrap();
        
        process_file(input_path, output_path, DEFAULT_KEY).unwrap();
        let encrypted = fs::read(output_path).unwrap();
        assert_ne!(encrypted, test_data);
        
        process_file(output_path, "test_decrypted.txt", DEFAULT_KEY).unwrap();
        let decrypted = fs::read("test_decrypted.txt").unwrap();
        assert_eq!(decrypted, test_data);
        
        fs::remove_file(input_path).unwrap();
        fs::remove_file(output_path).unwrap();
        fs::remove_file("test_decrypted.txt").unwrap();
    }
}
use std::fs;
use std::io::{self, Read, Write};

pub fn xor_encrypt_file(input_path: &str, output_path: &str, key: &[u8]) -> io::Result<()> {
    let mut input_file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    input_file.read_to_end(&mut buffer)?;

    let encrypted_data: Vec<u8> = buffer
        .iter()
        .enumerate()
        .map(|(i, &byte)| byte ^ key[i % key.len()])
        .collect();

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&encrypted_data)?;

    Ok(())
}

pub fn xor_decrypt_file(input_path: &str, output_path: &str, key: &[u8]) -> io::Result<()> {
    xor_encrypt_file(input_path, output_path, key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_xor_encryption() {
        let test_data = b"Hello, Rust!";
        let key = b"secret";
        let temp_input = "test_input.txt";
        let temp_encrypted = "test_encrypted.txt";
        let temp_decrypted = "test_decrypted.txt";

        fs::write(temp_input, test_data).unwrap();

        xor_encrypt_file(temp_input, temp_encrypted, key).unwrap();
        xor_decrypt_file(temp_encrypted, temp_decrypted, key).unwrap();

        let decrypted_data = fs::read(temp_decrypted).unwrap();
        assert_eq!(decrypted_data, test_data);

        fs::remove_file(temp_input).unwrap_or_default();
        fs::remove_file(temp_encrypted).unwrap_or_default();
        fs::remove_file(temp_decrypted).unwrap_or_default();
    }
}