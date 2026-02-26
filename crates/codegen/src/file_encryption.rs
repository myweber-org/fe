
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: u8 = 0x55;

pub fn encrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    let encryption_key = key.unwrap_or(DEFAULT_KEY);
    
    let mut input_file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    input_file.read_to_end(&mut buffer)?;
    
    for byte in &mut buffer {
        *byte ^= encryption_key;
    }
    
    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&buffer)?;
    
    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    encrypt_file(input_path, output_path, key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_encryption_decryption() {
        let original_content = b"Hello, World! This is a test file.";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), original_content).unwrap();
        
        encrypt_file(
            input_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap(),
            Some(0x42)
        ).unwrap();
        
        let encrypted_content = fs::read(encrypted_file.path()).unwrap();
        assert_ne!(encrypted_content, original_content);
        
        decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap(),
            Some(0x42)
        ).unwrap();
        
        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(decrypted_content, original_content);
    }
    
    #[test]
    fn test_default_key() {
        let original_content = b"Test with default key";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), original_content).unwrap();
        
        encrypt_file(
            input_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap(),
            None
        ).unwrap();
        
        decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap(),
            None
        ).unwrap();
        
        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(decrypted_content, original_content);
    }
}
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use std::fs;

pub fn encrypt_file(input_path: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let data = fs::read(input_path)?;
    
    let key = Aes256Gcm::generate_key(&mut OsRng);
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(b"unique_nonce_");
    
    let ciphertext = cipher.encrypt(nonce, data.as_ref())
        .map_err(|e| format!("Encryption failed: {}", e))?;
    
    let mut output = key.to_vec();
    output.extend_from_slice(nonce);
    output.extend_from_slice(&ciphertext);
    
    fs::write(output_path, output)?;
    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let encrypted_data = fs::read(input_path)?;
    
    if encrypted_data.len() < 48 {
        return Err("Invalid encrypted file format".into());
    }
    
    let key = &encrypted_data[..32];
    let nonce = &encrypted_data[32..44];
    let ciphertext = &encrypted_data[44..];
    
    let cipher = Aes256Gcm::new_from_slice(key)?;
    let nonce = Nonce::from_slice(nonce);
    
    let plaintext = cipher.decrypt(nonce, ciphertext)
        .map_err(|e| format!("Decryption failed: {}", e))?;
    
    fs::write(output_path, plaintext)?;
    Ok(())
}