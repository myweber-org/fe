
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use argon2::{
    password_hash::{rand_core::OsRng as ArgonRng, PasswordHasher, SaltString},
    Argon2
};
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const NONCE_SIZE: usize = 12;
const SALT_SIZE: usize = 16;

pub struct FileEncryptor {
    cipher: Aes256Gcm,
}

impl FileEncryptor {
    pub fn new(password: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let salt = SaltString::generate(&mut ArgonRng);
        let argon2 = Argon2::default();
        
        let password_hash = argon2.hash_password(password.as_bytes(), &salt)?;
        let key_bytes = password_hash.hash.ok_or("Hash generation failed")?;
        
        let key = Key::<Aes256Gcm>::from_slice(key_bytes.as_bytes());
        let cipher = Aes256Gcm::new(key);
        
        Ok(Self { cipher })
    }
    
    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let mut input_file = fs::File::open(input_path)?;
        let mut plaintext = Vec::new();
        input_file.read_to_end(&mut plaintext)?;
        
        let mut nonce_bytes = [0u8; NONCE_SIZE];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        let ciphertext = self.cipher.encrypt(nonce, plaintext.as_ref())
            .map_err(|e| format!("Encryption failed: {}", e))?;
        
        let mut output_file = fs::File::create(output_path)?;
        output_file.write_all(&nonce_bytes)?;
        output_file.write_all(&ciphertext)?;
        
        Ok(())
    }
    
    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let mut input_file = fs::File::open(input_path)?;
        let mut encrypted_data = Vec::new();
        input_file.read_to_end(&mut encrypted_data)?;
        
        if encrypted_data.len() < NONCE_SIZE {
            return Err("Invalid encrypted file format".into());
        }
        
        let (nonce_bytes, ciphertext) = encrypted_data.split_at(NONCE_SIZE);
        let nonce = Nonce::from_slice(nonce_bytes);
        
        let plaintext = self.cipher.decrypt(nonce, ciphertext)
            .map_err(|e| format!("Decryption failed: {}", e))?;
        
        let mut output_file = fs::File::create(output_path)?;
        output_file.write_all(&plaintext)?;
        
        Ok(())
    }
}

pub fn process_encryption() -> Result<(), Box<dyn std::error::Error>> {
    let password = "secure_passphrase_123";
    let encryptor = FileEncryptor::new(password)?;
    
    let test_data = b"Confidential data: API keys, database credentials, and secret tokens";
    let input_path = Path::new("test_input.txt");
    let encrypted_path = Path::new("test_encrypted.bin");
    let decrypted_path = Path::new("test_decrypted.txt");
    
    fs::write(input_path, test_data)?;
    
    encryptor.encrypt_file(input_path, encrypted_path)?;
    println!("File encrypted successfully");
    
    encryptor.decrypt_file(encrypted_path, decrypted_path)?;
    println!("File decrypted successfully");
    
    let decrypted_content = fs::read(decrypted_path)?;
    assert_eq!(test_data.as_ref(), decrypted_content.as_slice());
    
    fs::remove_file(input_path)?;
    fs::remove_file(encrypted_path)?;
    fs::remove_file(decrypted_path)?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_encryption_decryption_cycle() {
        let password = "test_password";
        let encryptor = FileEncryptor::new(password).unwrap();
        
        let original_content = b"Sample confidential data for testing";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), original_content).unwrap();
        
        encryptor.encrypt_file(input_file.path(), encrypted_file.path()).unwrap();
        encryptor.decrypt_file(encrypted_file.path(), decrypted_file.path()).unwrap();
        
        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(original_content.as_ref(), decrypted_content.as_slice());
    }
    
    #[test]
    fn test_wrong_password_fails() {
        let encryptor1 = FileEncryptor::new("password1").unwrap();
        let encryptor2 = FileEncryptor::new("password2").unwrap();
        
        let test_content = b"Secret message";
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), test_content).unwrap();
        encryptor1.encrypt_file(input_file.path(), encrypted_file.path()).unwrap();
        
        let result = encryptor2.decrypt_file(encrypted_file.path(), NamedTempFile::new().unwrap().path());
        assert!(result.is_err());
    }
}