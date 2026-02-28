
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: u8 = 0x55;

pub fn xor_encrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    let encryption_key = key.unwrap_or(DEFAULT_KEY);
    
    let input_data = fs::read(input_path)?;
    
    let encrypted_data: Vec<u8> = input_data
        .iter()
        .map(|byte| byte ^ encryption_key)
        .collect();
    
    fs::write(output_path, encrypted_data)?;
    
    Ok(())
}

pub fn xor_decrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    xor_encrypt_file(input_path, output_path, key)
}

pub fn process_file_interactive() -> io::Result<()> {
    println!("File Encryption Utility");
    println!("=======================");
    
    let mut input_path = String::new();
    print!("Enter input file path: ");
    io::stdout().flush()?;
    io::stdin().read_line(&mut input_path)?;
    let input_path = input_path.trim();
    
    let mut output_path = String::new();
    print!("Enter output file path: ");
    io::stdout().flush()?;
    io::stdin().read_line(&mut output_path)?;
    let output_path = output_path.trim();
    
    println!("Choose operation:");
    println!("1. Encrypt");
    println!("2. Decrypt");
    
    let mut choice = String::new();
    io::stdin().read_line(&mut choice)?;
    
    let mut key_input = String::new();
    print!("Enter encryption key (0-255, empty for default): ");
    io::stdout().flush()?;
    io::stdin().read_line(&mut key_input)?;
    
    let key = if key_input.trim().is_empty() {
        None
    } else {
        match key_input.trim().parse::<u8>() {
            Ok(k) => Some(k),
            Err(_) => {
                println!("Invalid key, using default");
                None
            }
        }
    };
    
    match choice.trim() {
        "1" => {
            xor_encrypt_file(input_path, output_path, key)?;
            println!("File encrypted successfully");
        }
        "2" => {
            xor_decrypt_file(input_path, output_path, key)?;
            println!("File decrypted successfully");
        }
        _ => {
            println!("Invalid choice");
            return Ok(());
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_xor_encryption() {
        let test_data = b"Hello, World!";
        let key = 0x42;
        
        let input_file = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), test_data).unwrap();
        
        xor_encrypt_file(
            input_file.path().to_str().unwrap(),
            output_file.path().to_str().unwrap(),
            Some(key)
        ).unwrap();
        
        let encrypted = fs::read(output_file.path()).unwrap();
        assert_ne!(encrypted, test_data);
        
        xor_decrypt_file(
            output_file.path().to_str().unwrap(),
            input_file.path().to_str().unwrap(),
            Some(key)
        ).unwrap();
        
        let decrypted = fs::read(input_file.path()).unwrap();
        assert_eq!(decrypted, test_data);
    }
}