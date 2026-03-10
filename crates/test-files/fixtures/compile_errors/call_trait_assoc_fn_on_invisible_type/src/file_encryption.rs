
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use pbkdf2::{pbkdf2_hmac, Params};
use sha2::Sha256;
use std::fs;
use std::path::Path;

const SALT_LENGTH: usize = 16;
const NONCE_LENGTH: usize = 12;
const PBKDF2_ITERATIONS: u32 = 100_000;

pub struct EncryptionResult {
    pub ciphertext: Vec<u8>,
    pub salt: [u8; SALT_LENGTH],
    pub nonce: [u8; NONCE_LENGTH],
}

pub fn encrypt_file(
    input_path: &Path,
    output_path: &Path,
    password: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let plaintext = fs::read(input_path)?;
    
    let mut salt = [0u8; SALT_LENGTH];
    OsRng.fill_bytes(&mut salt);
    
    let mut key = [0u8; 32];
    pbkdf2_hmac::<Sha256>(
        password.as_bytes(),
        &salt,
        PBKDF2_ITERATIONS,
        &mut key,
        Params {
            rounds: PBKDF2_ITERATIONS,
            output_length: 32,
        },
    );
    
    let mut nonce = [0u8; NONCE_LENGTH];
    OsRng.fill_bytes(&mut nonce);
    
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key));
    let ciphertext = cipher
        .encrypt(Nonce::from_slice(&nonce), plaintext.as_ref())
        .map_err(|e| format!("Encryption failed: {}", e))?;
    
    let result = EncryptionResult {
        ciphertext,
        salt,
        nonce,
    };
    
    let mut output_data = Vec::new();
    output_data.extend_from_slice(&result.salt);
    output_data.extend_from_slice(&result.nonce);
    output_data.extend_from_slice(&result.ciphertext);
    
    fs::write(output_path, output_data)?;
    Ok(())
}

pub fn decrypt_file(
    input_path: &Path,
    output_path: &Path,
    password: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let encrypted_data = fs::read(input_path)?;
    
    if encrypted_data.len() < SALT_LENGTH + NONCE_LENGTH {
        return Err("Invalid encrypted file format".into());
    }
    
    let salt = &encrypted_data[..SALT_LENGTH];
    let nonce = &encrypted_data[SALT_LENGTH..SALT_LENGTH + NONCE_LENGTH];
    let ciphertext = &encrypted_data[SALT_LENGTH + NONCE_LENGTH..];
    
    let mut key = [0u8; 32];
    pbkdf2_hmac::<Sha256>(
        password.as_bytes(),
        salt,
        PBKDF2_ITERATIONS,
        &mut key,
        Params {
            rounds: PBKDF2_ITERATIONS,
            output_length: 32,
        },
    );
    
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key));
    let plaintext = cipher
        .decrypt(Nonce::from_slice(nonce), ciphertext)
        .map_err(|e| format!("Decryption failed: {}", e))?;
    
    fs::write(output_path, plaintext)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let plaintext = b"Secret data that needs protection";
        let password = "strong_password_123";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), plaintext).unwrap();
        
        encrypt_file(input_file.path(), encrypted_file.path(), password)
            .expect("Encryption should succeed");
        
        decrypt_file(encrypted_file.path(), decrypted_file.path(), password)
            .expect("Decryption should succeed");
        
        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(plaintext.to_vec(), decrypted_data);
    }
    
    #[test]
    fn test_wrong_password_fails() {
        let plaintext = b"Test data";
        let password = "correct_password";
        let wrong_password = "wrong_password";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), plaintext).unwrap();
        
        encrypt_file(input_file.path(), encrypted_file.path(), password)
            .expect("Encryption should succeed");
        
        let result = decrypt_file(encrypted_file.path(), decrypted_file.path(), wrong_password);
        assert!(result.is_err());
    }
}