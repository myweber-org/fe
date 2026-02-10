
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_encryption_decryption() {
        let original_text = b"Hello, Rust!";
        let test_key = Some(0xAA);
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), original_text).unwrap();
        
        xor_encrypt_file(
            input_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap(),
            test_key
        ).unwrap();
        
        xor_decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap(),
            test_key
        ).unwrap();
        
        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(decrypted_data, original_text);
    }
    
    #[test]
    fn test_default_key() {
        let original_text = b"Test with default key";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), original_text).unwrap();
        
        xor_encrypt_file(
            input_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap(),
            None
        ).unwrap();
        
        xor_decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap(),
            None
        ).unwrap();
        
        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(decrypted_data, original_text);
    }
}
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
use std::io::{Read, Write};
use std::path::Path;

const NONCE_SIZE: usize = 12;
const SALT_SIZE: usize = 16;

pub struct FileEncryptor {
    algorithm: String,
    version: u8,
}

impl FileEncryptor {
    pub fn new() -> Self {
        FileEncryptor {
            algorithm: "AES-256-GCM".to_string(),
            version: 1,
        }
    }

    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path, password: &str) -> Result<(), String> {
        let mut file_data = Vec::new();
        let mut file = fs::File::open(input_path)
            .map_err(|e| format!("Failed to open input file: {}", e))?;
        
        file.read_to_end(&mut file_data)
            .map_err(|e| format!("Failed to read file: {}", e))?;

        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        
        let password_hash = argon2.hash_password(password.as_bytes(), &salt)
            .map_err(|e| format!("Password hashing failed: {}", e))?;
        
        let key_material = password_hash.hash.ok_or("Hash generation failed")?;
        let key_bytes = key_material.as_bytes();
        
        if key_bytes.len() < 32 {
            return Err("Derived key too short".to_string());
        }
        
        let key_slice = &key_bytes[..32];
        let key = Key::<Aes256Gcm>::from_slice(key_slice);
        let cipher = Aes256Gcm::new(key);
        
        let mut nonce_bytes = [0u8; NONCE_SIZE];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        let ciphertext = cipher.encrypt(nonce, file_data.as_ref())
            .map_err(|e| format!("Encryption failed: {}", e))?;
        
        let mut output_data = Vec::new();
        output_data.extend_from_slice(salt.as_bytes());
        output_data.extend_from_slice(&nonce_bytes);
        output_data.extend_from_slice(&ciphertext);
        
        let mut output_file = fs::File::create(output_path)
            .map_err(|e| format!("Failed to create output file: {}", e))?;
        
        output_file.write_all(&output_data)
            .map_err(|e| format!("Failed to write encrypted data: {}", e))?;
        
        Ok(())
    }

    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path, password: &str) -> Result<(), String> {
        let mut encrypted_data = Vec::new();
        let mut file = fs::File::open(input_path)
            .map_err(|e| format!("Failed to open encrypted file: {}", e))?;
        
        file.read_to_end(&mut encrypted_data)
            .map_err(|e| format!("Failed to read encrypted data: {}", e))?;
        
        if encrypted_data.len() < SALT_SIZE + NONCE_SIZE {
            return Err("Invalid encrypted file format".to_string());
        }
        
        let salt_bytes = &encrypted_data[..SALT_SIZE];
        let salt_str = std::str::from_utf8(salt_bytes)
            .map_err(|e| format!("Invalid salt encoding: {}", e))?;
        let salt = SaltString::new(salt_str)
            .map_err(|e| format!("Invalid salt: {}", e))?;
        
        let nonce_start = SALT_SIZE;
        let nonce_end = nonce_start + NONCE_SIZE;
        let nonce_bytes = &encrypted_data[nonce_start..nonce_end];
        let ciphertext = &encrypted_data[nonce_end..];
        
        let argon2 = Argon2::default();
        let password_hash = argon2.hash_password(password.as_bytes(), &salt)
            .map_err(|e| format!("Password verification failed: {}", e))?;
        
        let key_material = password_hash.hash.ok_or("Hash extraction failed")?;
        let key_bytes = key_material.as_bytes();
        
        if key_bytes.len() < 32 {
            return Err("Derived key too short".to_string());
        }
        
        let key_slice = &key_bytes[..32];
        let key = Key::<Aes256Gcm>::from_slice(key_slice);
        let cipher = Aes256Gcm::new(key);
        let nonce = Nonce::from_slice(nonce_bytes);
        
        let plaintext = cipher.decrypt(nonce, ciphertext)
            .map_err(|e| format!("Decryption failed: {}", e))?;
        
        let mut output_file = fs::File::create(output_path)
            .map_err(|e| format!("Failed to create output file: {}", e))?;
        
        output_file.write_all(&plaintext)
            .map_err(|e| format!("Failed to write decrypted data: {}", e))?;
        
        Ok(())
    }

    pub fn get_algorithm(&self) -> &str {
        &self.algorithm
    }

    pub fn get_version(&self) -> u8 {
        self.version
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_encryption_decryption() {
        let encryptor = FileEncryptor::new();
        let test_data = b"Secret data that needs protection";
        
        let mut input_file = NamedTempFile::new().unwrap();
        input_file.write_all(test_data).unwrap();
        
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        let password = "strong_password_123";
        
        let encrypt_result = encryptor.encrypt_file(
            input_file.path(),
            encrypted_file.path(),
            password
        );
        assert!(encrypt_result.is_ok());
        
        let decrypt_result = encryptor.decrypt_file(
            encrypted_file.path(),
            decrypted_file.path(),
            password
        );
        assert!(decrypt_result.is_ok());
        
        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(decrypted_data, test_data);
    }

    #[test]
    fn test_wrong_password() {
        let encryptor = FileEncryptor::new();
        let test_data = b"Test data";
        
        let mut input_file = NamedTempFile::new().unwrap();
        input_file.write_all(test_data).unwrap();
        
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        let encrypt_result = encryptor.encrypt_file(
            input_file.path(),
            encrypted_file.path(),
            "correct_password"
        );
        assert!(encrypt_result.is_ok());
        
        let decrypt_result = encryptor.decrypt_file(
            encrypted_file.path(),
            decrypted_file.path(),
            "wrong_password"
        );
        assert!(decrypt_result.is_err());
    }
}