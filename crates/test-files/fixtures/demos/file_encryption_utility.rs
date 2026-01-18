use aes::Aes256;
use block_modes::{BlockMode, Cbc};
use block_modes::block_padding::Pkcs7;
use rand::RngCore;
use sha2::{Sha256, Digest};
use std::fs;
use std::io::{Read, Write};
use std::path::Path;

type Aes256Cbc = Cbc<Aes256, Pkcs7>;

const SALT_LENGTH: usize = 16;
const IV_LENGTH: usize = 16;
const KEY_ITERATIONS: u32 = 100_000;

pub fn derive_key(password: &str, salt: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    hasher.update(salt);
    
    for _ in 0..KEY_ITERATIONS {
        let result = hasher.finalize_reset();
        hasher.update(&result);
    }
    
    hasher.finalize().to_vec()
}

pub fn encrypt_file(input_path: &Path, output_path: &Path, password: &str) -> Result<(), String> {
    let mut rng = rand::thread_rng();
    let mut salt = [0u8; SALT_LENGTH];
    let mut iv = [0u8; IV_LENGTH];
    
    rng.fill_bytes(&mut salt);
    rng.fill_bytes(&mut iv);
    
    let key = derive_key(password, &salt);
    let cipher = Aes256Cbc::new_from_slices(&key, &iv)
        .map_err(|e| format!("Cipher initialization failed: {}", e))?;
    
    let mut input_data = fs::read(input_path)
        .map_err(|e| format!("Failed to read input file: {}", e))?;
    
    let encrypted_data = cipher.encrypt_vec(&mut input_data);
    
    let mut output_data = Vec::with_capacity(SALT_LENGTH + IV_LENGTH + encrypted_data.len());
    output_data.extend_from_slice(&salt);
    output_data.extend_from_slice(&iv);
    output_data.extend_from_slice(&encrypted_data);
    
    fs::write(output_path, output_data)
        .map_err(|e| format!("Failed to write output file: {}", e))?;
    
    Ok(())
}

pub fn decrypt_file(input_path: &Path, output_path: &Path, password: &str) -> Result<(), String> {
    let encrypted_data = fs::read(input_path)
        .map_err(|e| format!("Failed to read encrypted file: {}", e))?;
    
    if encrypted_data.len() < SALT_LENGTH + IV_LENGTH {
        return Err("Invalid encrypted file format".to_string());
    }
    
    let salt = &encrypted_data[0..SALT_LENGTH];
    let iv = &encrypted_data[SALT_LENGTH..SALT_LENGTH + IV_LENGTH];
    let ciphertext = &encrypted_data[SALT_LENGTH + IV_LENGTH..];
    
    let key = derive_key(password, salt);
    let cipher = Aes256Cbc::new_from_slices(&key, iv)
        .map_err(|e| format!("Cipher initialization failed: {}", e))?;
    
    let decrypted_data = cipher.decrypt_vec(ciphertext)
        .map_err(|e| format!("Decryption failed: {}", e))?;
    
    fs::write(output_path, decrypted_data)
        .map_err(|e| format!("Failed to write decrypted file: {}", e))?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_encryption_decryption() {
        let original_content = b"Secret data that needs protection";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), original_content).unwrap();
        
        let password = "strong_password_123";
        
        encrypt_file(input_file.path(), encrypted_file.path(), password)
            .expect("Encryption should succeed");
        
        decrypt_file(encrypted_file.path(), decrypted_file.path(), password)
            .expect("Decryption should succeed");
        
        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(original_content.to_vec(), decrypted_content);
    }
    
    #[test]
    fn test_wrong_password() {
        let original_content = b"Test data";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), original_content).unwrap();
        
        encrypt_file(input_file.path(), encrypted_file.path(), "correct_password")
            .expect("Encryption should succeed");
        
        let result = decrypt_file(encrypted_file.path(), decrypted_file.path(), "wrong_password");
        assert!(result.is_err());
    }
}