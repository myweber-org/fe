
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use argon2::{Argon2, PasswordHasher, PasswordVerifier};
use rand::RngCore;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;

const NONCE_LENGTH: usize = 12;
const SALT_LENGTH: usize = 16;
const TAG_LENGTH: usize = 16;

pub struct EncryptionResult {
    pub ciphertext: Vec<u8>,
    pub nonce: [u8; NONCE_LENGTH],
    pub salt: [u8; SALT_LENGTH],
}

pub fn derive_key(password: &str, salt: &[u8]) -> Result<Key<Aes256Gcm>, String> {
    let argon2 = Argon2::default();
    let mut key = [0u8; 32];
    
    argon2
        .hash_password_into(password.as_bytes(), salt, &mut key)
        .map_err(|e| format!("Key derivation failed: {}", e))?;
    
    Ok(Key::<Aes256Gcm>::from_slice(&key).clone())
}

pub fn encrypt_file(
    input_path: &Path,
    output_path: &Path,
    password: &str,
) -> Result<EncryptionResult, String> {
    let mut input_file = File::open(input_path)
        .map_err(|e| format!("Failed to open input file: {}", e))?;
    
    let mut plaintext = Vec::new();
    input_file
        .read_to_end(&mut plaintext)
        .map_err(|e| format!("Failed to read input file: {}", e))?;
    
    let mut rng = rand::thread_rng();
    let mut salt = [0u8; SALT_LENGTH];
    let mut nonce_bytes = [0u8; NONCE_LENGTH];
    
    rng.fill_bytes(&mut salt);
    rng.fill_bytes(&mut nonce_bytes);
    
    let key = derive_key(password, &salt)?;
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(&nonce_bytes);
    
    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_ref())
        .map_err(|e| format!("Encryption failed: {}", e))?;
    
    let mut output_file = File::create(output_path)
        .map_err(|e| format!("Failed to create output file: {}", e))?;
    
    output_file
        .write_all(&salt)
        .map_err(|e| format!("Failed to write salt: {}", e))?;
    output_file
        .write_all(&nonce_bytes)
        .map_err(|e| format!("Failed to write nonce: {}", e))?;
    output_file
        .write_all(&ciphertext)
        .map_err(|e| format!("Failed to write ciphertext: {}", e))?;
    
    Ok(EncryptionResult {
        ciphertext,
        nonce: nonce_bytes,
        salt,
    })
}

pub fn decrypt_file(
    input_path: &Path,
    output_path: &Path,
    password: &str,
) -> Result<Vec<u8>, String> {
    let mut input_file = File::open(input_path)
        .map_err(|e| format!("Failed to open input file: {}", e))?;
    
    let mut encrypted_data = Vec::new();
    input_file
        .read_to_end(&mut encrypted_data)
        .map_err(|e| format!("Failed to read input file: {}", e))?;
    
    if encrypted_data.len() < SALT_LENGTH + NONCE_LENGTH + TAG_LENGTH {
        return Err("File too short to contain valid encrypted data".to_string());
    }
    
    let salt = &encrypted_data[0..SALT_LENGTH];
    let nonce_bytes = &encrypted_data[SALT_LENGTH..SALT_LENGTH + NONCE_LENGTH];
    let ciphertext = &encrypted_data[SALT_LENGTH + NONCE_LENGTH..];
    
    let key = derive_key(password, salt)?;
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(nonce_bytes);
    
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| format!("Decryption failed: {}", e))?;
    
    let mut output_file = File::create(output_path)
        .map_err(|e| format!("Failed to create output file: {}", e))?;
    
    output_file
        .write_all(&plaintext)
        .map_err(|e| format!("Failed to write decrypted data: {}", e))?;
    
    Ok(plaintext)
}

pub fn generate_random_file(path: &Path, size: usize) -> Result<(), String> {
    let mut file = File::create(path)
        .map_err(|e| format!("Failed to create test file: {}", e))?;
    
    let mut rng = rand::thread_rng();
    let mut buffer = vec![0u8; size];
    rng.fill_bytes(&mut buffer);
    
    file.write_all(&buffer)
        .map_err(|e| format!("Failed to write test data: {}", e))?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_encryption_decryption() {
        let plaintext_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        let test_data = b"Test encryption and decryption data";
        fs::write(plaintext_file.path(), test_data).unwrap();
        
        let password = "secure_password_123";
        
        let encrypt_result = encrypt_file(
            plaintext_file.path(),
            encrypted_file.path(),
            password,
        );
        assert!(encrypt_result.is_ok());
        
        let decrypt_result = decrypt_file(
            encrypted_file.path(),
            decrypted_file.path(),
            password,
        );
        assert!(decrypt_result.is_ok());
        
        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(decrypted_data, test_data);
    }
    
    #[test]
    fn test_wrong_password() {
        let plaintext_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        let test_data = b"Sensitive information";
        fs::write(plaintext_file.path(), test_data).unwrap();
        
        let encrypt_result = encrypt_file(
            plaintext_file.path(),
            encrypted_file.path(),
            "correct_password",
        );
        assert!(encrypt_result.is_ok());
        
        let decrypt_result = decrypt_file(
            encrypted_file.path(),
            decrypted_file.path(),
            "wrong_password",
        );
        assert!(decrypt_result.is_err());
    }
}