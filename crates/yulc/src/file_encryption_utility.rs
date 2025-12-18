
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use pbkdf2::{
    password_hash::{
        rand_core::RngCore,
        PasswordHasher, SaltString
    },
    Pbkdf2
};
use std::fs;
use std::io::{Read, Write};
use std::path::Path;

pub struct FileEncryptor {
    algorithm: String,
    key_derivation_iterations: u32,
}

impl FileEncryptor {
    pub fn new() -> Self {
        FileEncryptor {
            algorithm: String::from("AES-256-GCM"),
            key_derivation_iterations: 100_000,
        }
    }

    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path, password: &str) -> Result<(), String> {
        let mut file_data = Vec::new();
        let mut file = fs::File::open(input_path)
            .map_err(|e| format!("Failed to open input file: {}", e))?;
        
        file.read_to_end(&mut file_data)
            .map_err(|e| format!("Failed to read file: {}", e))?;

        let salt = SaltString::generate(&mut OsRng);
        let password_hash = Pbkdf2.hash_password(password.as_bytes(), &salt)
            .map_err(|e| format!("Key derivation failed: {}", e))?;
        
        let hash_bytes = password_hash.hash.ok_or("No hash generated")?.as_bytes();
        let mut key_bytes = [0u8; 32];
        key_bytes.copy_from_slice(&hash_bytes[..32]);

        let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
        let cipher = Aes256Gcm::new(key);
        
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher.encrypt(nonce, file_data.as_ref())
            .map_err(|e| format!("Encryption failed: {}", e))?;

        let mut output_data = Vec::new();
        output_data.extend_from_slice(salt.as_str().as_bytes());
        output_data.push(b'|');
        output_data.extend_from_slice(&nonce_bytes);
        output_data.push(b'|');
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

        let parts: Vec<&[u8]> = encrypted_data.split(|&b| b == b'|').collect();
        if parts.len() != 3 {
            return Err("Invalid encrypted file format".to_string());
        }

        let salt_str = std::str::from_utf8(parts[0])
            .map_err(|_| "Invalid salt encoding")?;
        let salt = SaltString::new(salt_str)
            .map_err(|e| format!("Invalid salt: {}", e))?;

        let password_hash = Pbkdf2.hash_password(password.as_bytes(), &salt)
            .map_err(|e| format!("Key derivation failed: {}", e))?;
        
        let hash_bytes = password_hash.hash.ok_or("No hash generated")?.as_bytes();
        let mut key_bytes = [0u8; 32];
        key_bytes.copy_from_slice(&hash_bytes[..32]);

        let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
        let cipher = Aes256Gcm::new(key);

        let nonce_bytes: [u8; 12] = parts[1].try_into()
            .map_err(|_| "Invalid nonce length")?;
        let nonce = Nonce::from_slice(&nonce_bytes);

        let plaintext = cipher.decrypt(nonce, parts[2])
            .map_err(|e| format!("Decryption failed: {}", e))?;

        let mut output_file = fs::File::create(output_path)
            .map_err(|e| format!("Failed to create output file: {}", e))?;
        
        output_file.write_all(&plaintext)
            .map_err(|e| format!("Failed to write decrypted data: {}", e))?;

        Ok(())
    }

    pub fn get_algorithm_info(&self) -> String {
        format!("{} with PBKDF2 key derivation ({} iterations)", 
                self.algorithm, self.key_derivation_iterations)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let encryptor = FileEncryptor::new();
        let test_data = b"Test data for encryption and decryption";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), test_data).unwrap();
        
        let password = "secure_password_123";
        
        encryptor.encrypt_file(input_file.path(), encrypted_file.path(), password)
            .expect("Encryption should succeed");
        
        encryptor.decrypt_file(encrypted_file.path(), decrypted_file.path(), password)
            .expect("Decryption should succeed");
        
        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(test_data.to_vec(), decrypted_data);
    }
}