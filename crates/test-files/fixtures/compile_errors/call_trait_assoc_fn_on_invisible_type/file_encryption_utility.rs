use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use std::{
    fs::{File, OpenOptions},
    io::{Read, Write},
    path::Path
};

pub struct FileEncryptor {
    cipher: Aes256Gcm
}

impl FileEncryptor {
    pub fn new() -> Self {
        let key = Aes256Gcm::generate_key(&mut OsRng);
        Self {
            cipher: Aes256Gcm::new(&key)
        }
    }

    pub fn encrypt_file(&self, source_path: &Path, dest_path: &Path) -> Result<(), String> {
        let mut source_file = File::open(source_path)
            .map_err(|e| format!("Failed to open source file: {}", e))?;
        
        let mut plaintext = Vec::new();
        source_file.read_to_end(&mut plaintext)
            .map_err(|e| format!("Failed to read source file: {}", e))?;

        let nonce = Nonce::from_slice(&[0u8; 12]);
        let ciphertext = self.cipher.encrypt(nonce, plaintext.as_ref())
            .map_err(|e| format!("Encryption failed: {}", e))?;

        let mut dest_file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(dest_path)
            .map_err(|e| format!("Failed to create destination file: {}", e))?;

        dest_file.write_all(&ciphertext)
            .map_err(|e| format!("Failed to write encrypted data: {}", e))?;

        Ok(())
    }

    pub fn decrypt_file(&self, source_path: &Path, dest_path: &Path) -> Result<(), String> {
        let mut source_file = File::open(source_path)
            .map_err(|e| format!("Failed to open encrypted file: {}", e))?;
        
        let mut ciphertext = Vec::new();
        source_file.read_to_end(&mut ciphertext)
            .map_err(|e| format!("Failed to read encrypted file: {}", e))?;

        let nonce = Nonce::from_slice(&[0u8; 12]);
        let plaintext = self.cipher.decrypt(nonce, ciphertext.as_ref())
            .map_err(|e| format!("Decryption failed: {}", e))?;

        let mut dest_file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(dest_path)
            .map_err(|e| format!("Failed to create output file: {}", e))?;

        dest_file.write_all(&plaintext)
            .map_err(|e| format!("Failed to write decrypted data: {}", e))?;

        Ok(())
    }
}

pub fn generate_random_key() -> Vec<u8> {
    let key = Aes256Gcm::generate_key(&mut OsRng);
    key.to_vec()
}