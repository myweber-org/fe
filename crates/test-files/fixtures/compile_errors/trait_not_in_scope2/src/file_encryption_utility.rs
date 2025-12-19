use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHasher, SaltString
    },
    Argon2
};
use std::fs;
use std::path::Path;

pub struct EncryptionResult {
    pub ciphertext: Vec<u8>,
    pub nonce: Vec<u8>,
    pub salt: String,
}

pub fn derive_key(password: &str, salt: &str) -> Vec<u8> {
    let salt_bytes = SaltString::from_b64(salt).unwrap();
    let argon2 = Argon2::default();
    let password_hash = argon2.hash_password(password.as_bytes(), &salt_bytes).unwrap();
    password_hash.hash.unwrap().as_bytes().to_vec()
}

pub fn encrypt_file(
    file_path: &Path,
    password: &str
) -> Result<EncryptionResult, Box<dyn std::error::Error>> {
    let plaintext = fs::read(file_path)?;
    
    let salt = SaltString::generate(&mut OsRng);
    let key_material = derive_key(password, salt.as_str());
    
    let key = Key::<Aes256Gcm>::from_slice(&key_material);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::generate(&mut OsRng);
    
    let ciphertext = cipher
        .encrypt(&nonce, plaintext.as_ref())
        .map_err(|e| format!("Encryption failed: {}", e))?;
    
    Ok(EncryptionResult {
        ciphertext,
        nonce: nonce.to_vec(),
        salt: salt.to_string(),
    })
}

pub fn decrypt_file(
    encrypted_data: &EncryptionResult,
    password: &str
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let key_material = derive_key(password, &encrypted_data.salt);
    let key = Key::<Aes256Gcm>::from_slice(&key_material);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(&encrypted_data.nonce);
    
    let plaintext = cipher
        .decrypt(nonce, encrypted_data.ciphertext.as_ref())
        .map_err(|e| format!("Decryption failed: {}", e))?;
    
    Ok(plaintext)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_roundtrip() {
        let test_content = b"Secret data that needs protection";
        let password = "strong_password_123";
        
        let mut temp_file = NamedTempFile::new().unwrap();
        std::fs::write(temp_file.path(), test_content).unwrap();
        
        let encrypted = encrypt_file(temp_file.path(), password).unwrap();
        let decrypted = decrypt_file(&encrypted, password).unwrap();
        
        assert_eq!(test_content.to_vec(), decrypted);
    }
    
    #[test]
    fn test_wrong_password_fails() {
        let test_content = b"Test data";
        let password = "correct_password";
        let wrong_password = "wrong_password";
        
        let mut temp_file = NamedTempFile::new().unwrap();
        std::fs::write(temp_file.path(), test_content).unwrap();
        
        let encrypted = encrypt_file(temp_file.path(), password).unwrap();
        let result = decrypt_file(&encrypted, wrong_password);
        
        assert!(result.is_err());
    }
}