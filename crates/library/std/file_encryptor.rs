
use std::fs;
use std::io::{Read, Write};
use std::path::Path;

const SALT: &[u8] = b"static_salt_for_key_derivation";

pub fn derive_key(password: &str, key_length: usize) -> Vec<u8> {
    let mut key = Vec::with_capacity(key_length);
    let mut hasher = blake3::Hasher::new();
    
    hasher.update(password.as_bytes());
    hasher.update(SALT);
    
    let mut output_reader = hasher.finalize_xof();
    let mut buffer = [0u8; 32];
    
    while key.len() < key_length {
        output_reader.fill(&mut buffer);
        key.extend_from_slice(&buffer[..key_length.min(key.len() + 32)]);
    }
    
    key.truncate(key_length);
    key
}

pub fn xor_cipher(data: &mut [u8], key: &[u8]) {
    for (i, byte) in data.iter_mut().enumerate() {
        *byte ^= key[i % key.len()];
    }
}

pub fn encrypt_file(input_path: &Path, output_path: &Path, password: &str) -> Result<(), String> {
    let mut file_data = fs::read(input_path)
        .map_err(|e| format!("Failed to read input file: {}", e))?;
    
    let key = derive_key(password, 32);
    xor_cipher(&mut file_data, &key);
    
    fs::write(output_path, &file_data)
        .map_err(|e| format!("Failed to write output file: {}", e))?;
    
    Ok(())
}

pub fn decrypt_file(input_path: &Path, output_path: &Path, password: &str) -> Result<(), String> {
    encrypt_file(input_path, output_path, password)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_symmetric_encryption() {
        let test_data = b"Hello, this is a secret message!";
        let password = "strong_password_123";
        
        let mut encrypted = test_data.to_vec();
        let key = derive_key(password, 32);
        xor_cipher(&mut encrypted, &key);
        
        assert_ne!(encrypted, test_data);
        
        xor_cipher(&mut encrypted, &key);
        assert_eq!(encrypted, test_data);
    }

    #[test]
    fn test_file_operations() {
        let input_file = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();
        let password = "test_pass";
        
        let original_content = b"File encryption test content";
        fs::write(input_file.path(), original_content).unwrap();
        
        encrypt_file(input_file.path(), output_file.path(), password)
            .expect("Encryption should succeed");
        
        let encrypted_content = fs::read(output_file.path()).unwrap();
        assert_ne!(encrypted_content, original_content);
        
        let decrypted_file = NamedTempFile::new().unwrap();
        decrypt_file(output_file.path(), decrypted_file.path(), password)
            .expect("Decryption should succeed");
        
        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(decrypted_content, original_content);
    }
}