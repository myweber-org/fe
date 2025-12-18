
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: u8 = 0xAA;

pub fn encrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    let encryption_key = key.unwrap_or(DEFAULT_KEY);
    
    let mut input_file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    input_file.read_to_end(&mut buffer)?;
    
    let encrypted_data: Vec<u8> = buffer
        .iter()
        .map(|byte| byte ^ encryption_key)
        .collect();
    
    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&encrypted_data)?;
    
    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    encrypt_file(input_path, output_path, key)
}

pub fn process_files() -> io::Result<()> {
    let test_data = b"Secret message for encryption test";
    let test_file = "test_input.bin";
    let encrypted_file = "test_encrypted.bin";
    let decrypted_file = "test_decrypted.bin";
    
    fs::write(test_file, test_data)?;
    
    println!("Encrypting file...");
    encrypt_file(test_file, encrypted_file, Some(0xCC))?;
    
    println!("Decrypting file...");
    decrypt_file(encrypted_file, decrypted_file, Some(0xCC))?;
    
    let decrypted_content = fs::read(decrypted_file)?;
    
    if decrypted_content == test_data {
        println!("Encryption/decryption successful!");
    } else {
        println!("Encryption/decryption failed!");
    }
    
    fs::remove_file(test_file)?;
    fs::remove_file(encrypted_file)?;
    fs::remove_file(decrypted_file)?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_xor_cipher() {
        let data = vec![0x00, 0xFF, 0x55, 0xAA];
        let key = 0xCC;
        
        let encrypted: Vec<u8> = data.iter().map(|byte| byte ^ key).collect();
        let decrypted: Vec<u8> = encrypted.iter().map(|byte| byte ^ key).collect();
        
        assert_eq!(data, decrypted);
    }
}