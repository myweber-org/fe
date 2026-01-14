
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
    key: [u8; 32],
}

impl FileEncryptor {
    pub fn from_password(password: &str, salt: &[u8]) -> Result<Self, String> {
        let argon2 = Argon2::default();
        let salt_string = SaltString::encode_b64(salt)
            .map_err(|e| format!("Salt encoding failed: {}", e))?;
        
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt_string)
            .map_err(|e| format!("Key derivation failed: {}", e))?;
        
        let hash_bytes = password_hash.hash.ok_or("No hash generated")?;
        let mut key = [0u8; 32];
        key.copy_from_slice(&hash_bytes.as_bytes()[..32]);
        
        Ok(Self { key })
    }
    
    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), String> {
        let mut file_data = Vec::new();
        fs::File::open(input_path)
            .map_err(|e| format!("Failed to open input file: {}", e))?
            .read_to_end(&mut file_data)
            .map_err(|e| format!("Failed to read file: {}", e))?;
        
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&self.key));
        let nonce = Nonce::from_slice(&generate_random_bytes(NONCE_SIZE));
        
        let encrypted_data = cipher
            .encrypt(nonce, file_data.as_ref())
            .map_err(|e| format!("Encryption failed: {}", e))?;
        
        let mut output = Vec::new();
        output.extend_from_slice(nonce.as_slice());
        output.extend_from_slice(&encrypted_data);
        
        fs::File::create(output_path)
            .map_err(|e| format!("Failed to create output file: {}", e))?
            .write_all(&output)
            .map_err(|e| format!("Failed to write encrypted data: {}", e))?;
        
        Ok(())
    }
    
    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), String> {
        let mut encrypted_data = Vec::new();
        fs::File::open(input_path)
            .map_err(|e| format!("Failed to open encrypted file: {}", e))?
            .read_to_end(&mut encrypted_data)
            .map_err(|e| format!("Failed to read encrypted data: {}", e))?;
        
        if encrypted_data.len() < NONCE_SIZE {
            return Err("Invalid encrypted file format".to_string());
        }
        
        let (nonce_bytes, ciphertext) = encrypted_data.split_at(NONCE_SIZE);
        let nonce = Nonce::from_slice(nonce_bytes);
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&self.key));
        
        let decrypted_data = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| format!("Decryption failed: {}", e))?;
        
        fs::File::create(output_path)
            .map_err(|e| format!("Failed to create output file: {}", e))?
            .write_all(&decrypted_data)
            .map_err(|e| format!("Failed to write decrypted data: {}", e))?;
        
        Ok(())
    }
}

fn generate_random_bytes(size: usize) -> Vec<u8> {
    let mut bytes = vec![0u8; size];
    OsRng.fill_bytes(&mut bytes);
    bytes
}

pub fn generate_salt() -> Vec<u8> {
    generate_random_bytes(SALT_SIZE)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_encryption_decryption() {
        let password = "secure_password_123";
        let salt = generate_salt();
        
        let encryptor = FileEncryptor::from_password(password, &salt)
            .expect("Failed to create encryptor");
        
        let test_data = b"Hello, this is a secret message!";
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), test_data).unwrap();
        
        encryptor.encrypt_file(input_file.path(), encrypted_file.path())
            .expect("Encryption failed");
        
        encryptor.decrypt_file(encrypted_file.path(), decrypted_file.path())
            .expect("Decryption failed");
        
        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(test_data.to_vec(), decrypted_data);
    }
}