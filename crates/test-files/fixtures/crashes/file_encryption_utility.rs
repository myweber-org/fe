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
    cipher: Aes256Gcm,
}

impl FileEncryptor {
    pub fn new(password: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2.hash_password(password.as_bytes(), &salt)?;
        
        let key_bytes = password_hash.hash.ok_or("Hash generation failed")?;
        let key = Key::<Aes256Gcm>::from_slice(key_bytes.as_bytes());
        let cipher = Aes256Gcm::new(key);
        
        Ok(FileEncryptor { cipher })
    }
    
    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let mut file_content = Vec::new();
        let mut file = fs::File::open(input_path)?;
        file.read_to_end(&mut file_content)?;
        
        let nonce = Nonce::generate(&mut OsRng);
        let ciphertext = self.cipher.encrypt(&nonce, file_content.as_ref())?;
        
        let mut output_file = fs::File::create(output_path)?;
        output_file.write_all(nonce.as_slice())?;
        output_file.write_all(&ciphertext)?;
        
        Ok(())
    }
    
    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let mut encrypted_data = Vec::new();
        let mut file = fs::File::open(input_path)?;
        file.read_to_end(&mut encrypted_data)?;
        
        if encrypted_data.len() < NONCE_SIZE {
            return Err("Invalid encrypted file format".into());
        }
        
        let (nonce_bytes, ciphertext) = encrypted_data.split_at(NONCE_SIZE);
        let nonce = Nonce::from_slice(nonce_bytes);
        let plaintext = self.cipher.decrypt(nonce, ciphertext)?;
        
        let mut output_file = fs::File::create(output_path)?;
        output_file.write_all(&plaintext)?;
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_encryption_decryption() {
        let password = "secure_password_123";
        let encryptor = FileEncryptor::new(password).unwrap();
        
        let original_content = b"Test data for encryption and decryption";
        let mut input_file = NamedTempFile::new().unwrap();
        input_file.write_all(original_content).unwrap();
        
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        encryptor.encrypt_file(input_file.path(), encrypted_file.path()).unwrap();
        encryptor.decrypt_file(encrypted_file.path(), decrypted_file.path()).unwrap();
        
        let mut decrypted_content = Vec::new();
        let mut decrypted = fs::File::open(decrypted_file.path()).unwrap();
        decrypted.read_to_end(&mut decrypted_content).unwrap();
        
        assert_eq!(original_content.to_vec(), decrypted_content);
    }
}use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use std::fs;
use std::io::{Read, Write};
use std::path::Path;

pub struct FileEncryptor {
    key: Key<Aes256Gcm>,
}

impl FileEncryptor {
    pub fn new() -> Self {
        let key = Aes256Gcm::generate_key(&mut OsRng);
        Self { key }
    }

    pub fn from_key(key: &[u8; 32]) -> Self {
        let key = Key::<Aes256Gcm>::from_slice(key);
        Self { key: *key }
    }

    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), String> {
        let cipher = Aes256Gcm::new(&self.key);
        let nonce = Nonce::from_slice(b"unique_nonce_12");

        let mut file = fs::File::open(input_path)
            .map_err(|e| format!("Failed to open input file: {}", e))?;
        
        let mut plaintext = Vec::new();
        file.read_to_end(&mut plaintext)
            .map_err(|e| format!("Failed to read file: {}", e))?;

        let ciphertext = cipher.encrypt(nonce, plaintext.as_ref())
            .map_err(|e| format!("Encryption failed: {}", e))?;

        let mut output_file = fs::File::create(output_path)
            .map_err(|e| format!("Failed to create output file: {}", e))?;
        
        output_file.write_all(&ciphertext)
            .map_err(|e| format!("Failed to write encrypted data: {}", e))?;

        Ok(())
    }

    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), String> {
        let cipher = Aes256Gcm::new(&self.key);
        let nonce = Nonce::from_slice(b"unique_nonce_12");

        let mut file = fs::File::open(input_path)
            .map_err(|e| format!("Failed to open encrypted file: {}", e))?;
        
        let mut ciphertext = Vec::new();
        file.read_to_end(&mut ciphertext)
            .map_err(|e| format!("Failed to read encrypted file: {}", e))?;

        let plaintext = cipher.decrypt(nonce, ciphertext.as_ref())
            .map_err(|e| format!("Decryption failed: {}", e))?;

        let mut output_file = fs::File::create(output_path)
            .map_err(|e| format!("Failed to create output file: {}", e))?;
        
        output_file.write_all(&plaintext)
            .map_err(|e| format!("Failed to write decrypted data: {}", e))?;

        Ok(())
    }

    pub fn export_key(&self) -> [u8; 32] {
        self.key.as_slice().try_into().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let encryptor = FileEncryptor::new();
        let test_data = b"Secret confidential data for encryption test";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        fs::write(input_file.path(), test_data).unwrap();

        encryptor.encrypt_file(input_file.path(), encrypted_file.path())
            .expect("Encryption should succeed");
        
        encryptor.decrypt_file(encrypted_file.path(), decrypted_file.path())
            .expect("Decryption should succeed");

        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(decrypted_data, test_data);
    }
}