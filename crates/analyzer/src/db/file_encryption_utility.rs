use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use argon2::{
    password_hash::{rand_core::OsRng as ArgonRng, PasswordHasher, SaltString},
    Argon2, Params,
};
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const SALT_LENGTH: usize = 16;
const NONCE_LENGTH: usize = 12;

pub struct EncryptionResult {
    pub encrypted_data: Vec<u8>,
    pub salt: [u8; SALT_LENGTH],
    pub nonce: [u8; NONCE_LENGTH],
}

pub fn derive_key(password: &str, salt: &[u8]) -> Result<Key<Aes256Gcm>, String> {
    let salt_str = SaltString::encode_b64(salt).map_err(|e| e.to_string())?;
    
    let params = Params::new(15_000, 2, 1, Some(32))
        .map_err(|e| e.to_string())?;
    
    let argon2 = Argon2::new(
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        params,
    );
    
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt_str)
        .map_err(|e| e.to_string())?;
    
    let hash_bytes = password_hash.hash.ok_or("Hash generation failed")?;
    let key_bytes: [u8; 32] = hash_bytes.as_bytes()[..32]
        .try_into()
        .map_err(|_| "Invalid hash length")?;
    
    Ok(Key::<Aes256Gcm>::from_slice(&key_bytes).clone())
}

pub fn encrypt_file(
    input_path: &Path,
    output_path: &Path,
    password: &str,
) -> Result<EncryptionResult, String> {
    let mut file = fs::File::open(input_path)
        .map_err(|e| format!("Failed to open input file: {}", e))?;
    
    let mut plaintext = Vec::new();
    file.read_to_end(&mut plaintext)
        .map_err(|e| format!("Failed to read file: {}", e))?;
    
    let mut salt = [0u8; SALT_LENGTH];
    ArgonRng.fill_bytes(&mut salt);
    
    let mut nonce_bytes = [0u8; NONCE_LENGTH];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    
    let key = derive_key(password, &salt)?;
    let cipher = Aes256Gcm::new(&key);
    
    let encrypted_data = cipher
        .encrypt(nonce, plaintext.as_ref())
        .map_err(|e| format!("Encryption failed: {}", e))?;
    
    let mut output_file = fs::File::create(output_path)
        .map_err(|e| format!("Failed to create output file: {}", e))?;
    
    output_file.write_all(&encrypted_data)
        .map_err(|e| format!("Failed to write encrypted data: {}", e))?;
    
    Ok(EncryptionResult {
        encrypted_data,
        salt,
        nonce: nonce_bytes,
    })
}

pub fn decrypt_file(
    input_path: &Path,
    output_path: &Path,
    password: &str,
    salt: &[u8; SALT_LENGTH],
    nonce: &[u8; NONCE_LENGTH],
) -> Result<Vec<u8>, String> {
    let mut file = fs::File::open(input_path)
        .map_err(|e| format!("Failed to open encrypted file: {}", e))?;
    
    let mut encrypted_data = Vec::new();
    file.read_to_end(&mut encrypted_data)
        .map_err(|e| format!("Failed to read encrypted data: {}", e))?;
    
    let key = derive_key(password, salt)?;
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(nonce);
    
    let decrypted_data = cipher
        .decrypt(nonce, encrypted_data.as_ref())
        .map_err(|e| format!("Decryption failed: {}", e))?;
    
    let mut output_file = fs::File::create(output_path)
        .map_err(|e| format!("Failed to create output file: {}", e))?;
    
    output_file.write_all(&decrypted_data)
        .map_err(|e| format!("Failed to write decrypted data: {}", e))?;
    
    Ok(decrypted_data)
}

pub fn encrypt_string(data: &str, password: &str) -> Result<EncryptionResult, String> {
    let mut salt = [0u8; SALT_LENGTH];
    ArgonRng.fill_bytes(&mut salt);
    
    let mut nonce_bytes = [0u8; NONCE_LENGTH];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    
    let key = derive_key(password, &salt)?;
    let cipher = Aes256Gcm::new(&key);
    
    let encrypted_data = cipher
        .encrypt(nonce, data.as_bytes())
        .map_err(|e| format!("String encryption failed: {}", e))?;
    
    Ok(EncryptionResult {
        encrypted_data,
        salt,
        nonce: nonce_bytes,
    })
}

pub fn decrypt_string(
    encrypted_data: &[u8],
    password: &str,
    salt: &[u8; SALT_LENGTH],
    nonce: &[u8; NONCE_LENGTH],
) -> Result<String, String> {
    let key = derive_key(password, salt)?;
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(nonce);
    
    let decrypted_bytes = cipher
        .decrypt(nonce, encrypted_data)
        .map_err(|e| format!("String decryption failed: {}", e))?;
    
    String::from_utf8(decrypted_bytes)
        .map_err(|e| format!("Invalid UTF-8 in decrypted data: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_string_encryption_decryption() {
        let original_text = "This is a secret message";
        let password = "strong_password_123";
        
        let encryption_result = encrypt_string(original_text, password).unwrap();
        
        let decrypted_text = decrypt_string(
            &encryption_result.encrypted_data,
            password,
            &encryption_result.salt,
            &encryption_result.nonce,
        ).unwrap();
        
        assert_eq!(original_text, decrypted_text);
    }

    #[test]
    fn test_file_encryption_decryption() {
        let original_content = b"File content to encrypt";
        let password = "another_secure_password";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), original_content).unwrap();
        
        let encryption_result = encrypt_file(
            input_file.path(),
            encrypted_file.path(),
            password,
        ).unwrap();
        
        let decrypted_content = decrypt_file(
            encrypted_file.path(),
            decrypted_file.path(),
            password,
            &encryption_result.salt,
            &encryption_result.nonce,
        ).unwrap();
        
        assert_eq!(original_content.to_vec(), decrypted_content);
    }

    #[test]
    fn test_wrong_password_fails() {
        let original_text = "Secret data";
        let correct_password = "correct_password";
        let wrong_password = "wrong_password";
        
        let encryption_result = encrypt_string(original_text, correct_password).unwrap();
        
        let result = decrypt_string(
            &encryption_result.encrypted_data,
            wrong_password,
            &encryption_result.salt,
            &encryption_result.nonce,
        );
        
        assert!(result.is_err());
    }
}