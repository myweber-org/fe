
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use pbkdf2::{
    password_hash::{
        PasswordHasher, SaltString, PasswordHash, PasswordVerifier
    },
    Pbkdf2
};
use rand_core::RngCore;
use std::fs;
use std::io::{self, Write};
use std::path::Path;

const SALT_LENGTH: usize = 16;
const NONCE_LENGTH: usize = 12;
const PBKDF2_ITERATIONS: u32 = 100_000;

pub struct EncryptionResult {
    pub ciphertext: Vec<u8>,
    pub nonce: [u8; NONCE_LENGTH],
    pub salt: [u8; SALT_LENGTH],
}

pub fn derive_key(password: &str, salt: &[u8]) -> Result<Key<Aes256Gcm>, String> {
    let salt_string = SaltString::encode_b64(salt)
        .map_err(|e| format!("Salt encoding failed: {}", e))?;
    
    let password_hash = Pbkdf2
        .hash_password(password.as_bytes(), &salt_string)
        .map_err(|e| format!("Key derivation failed: {}", e))?;
    
    let hash_bytes = password_hash.hash.ok_or("No hash generated")?.as_bytes();
    if hash_bytes.len() < 32 {
        return Err("Derived key too short".to_string());
    }
    
    let key_bytes: [u8; 32] = hash_bytes[..32].try_into()
        .map_err(|_| "Key conversion failed".to_string())?;
    
    Ok(Key::<Aes256Gcm>::from_slice(&key_bytes).clone())
}

pub fn encrypt_file(
    input_path: &Path,
    output_path: &Path,
    password: &str
) -> Result<EncryptionResult, String> {
    let plaintext = fs::read(input_path)
        .map_err(|e| format!("Failed to read input file: {}", e))?;
    
    let mut salt = [0u8; SALT_LENGTH];
    OsRng.fill_bytes(&mut salt);
    
    let key = derive_key(password, &salt)?;
    
    let cipher = Aes256Gcm::new(&key);
    
    let mut nonce_bytes = [0u8; NONCE_LENGTH];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    
    let ciphertext = cipher.encrypt(nonce, plaintext.as_ref())
        .map_err(|e| format!("Encryption failed: {}", e))?;
    
    fs::write(output_path, &ciphertext)
        .map_err(|e| format!("Failed to write output file: {}", e))?;
    
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
    nonce: &[u8; NONCE_LENGTH],
    salt: &[u8; SALT_LENGTH]
) -> Result<Vec<u8>, String> {
    let ciphertext = fs::read(input_path)
        .map_err(|e| format!("Failed to read encrypted file: {}", e))?;
    
    let key = derive_key(password, salt)?;
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(nonce);
    
    let plaintext = cipher.decrypt(nonce, ciphertext.as_ref())
        .map_err(|e| format!("Decryption failed: {}", e))?;
    
    fs::write(output_path, &plaintext)
        .map_err(|e| format!("Failed to write decrypted file: {}", e))?;
    
    Ok(plaintext)
}

pub fn verify_password(password: &str, salt: &[u8], test_hash: &str) -> bool {
    let salt_string = match SaltString::encode_b64(salt) {
        Ok(s) => s,
        Err(_) => return false,
    };
    
    let parsed_hash = match PasswordHash::new(test_hash) {
        Ok(h) => h,
        Err(_) => return false,
    };
    
    Pbkdf2.verify_password(password.as_bytes(), &parsed_hash).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let test_data = b"Test encryption data for AES-256-GCM";
        let password = "secure_password_123";
        
        let mut input_file = NamedTempFile::new().unwrap();
        input_file.write_all(test_data).unwrap();
        
        let output_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        let result = encrypt_file(
            input_file.path(),
            output_file.path(),
            password
        ).unwrap();
        
        let decrypted = decrypt_file(
            output_file.path(),
            decrypted_file.path(),
            password,
            &result.nonce,
            &result.salt
        ).unwrap();
        
        assert_eq!(decrypted, test_data);
    }
    
    #[test]
    fn test_password_verification() {
        let password = "test_password";
        let mut salt = [0u8; SALT_LENGTH];
        OsRng.fill_bytes(&mut salt);
        
        let key = derive_key(password, &salt).unwrap();
        let test_hash = format!("{:?}", key);
        
        assert!(verify_password(password, &salt, &test_hash));
        assert!(!verify_password("wrong_password", &salt, &test_hash));
    }
}