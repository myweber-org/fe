
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use std::fs;
use std::io::{self, Write};

pub struct FileEncryptor {
    cipher: Aes256Gcm,
}

impl FileEncryptor {
    pub fn new(key: &[u8; 32]) -> Self {
        let key = Key::<Aes256Gcm>::from_slice(key);
        let cipher = Aes256Gcm::new(key);
        FileEncryptor { cipher }
    }

    pub fn generate_key() -> [u8; 32] {
        let mut key = [0u8; 32];
        OsRng.fill_bytes(&mut key);
        key
    }

    pub fn encrypt_file(&self, input_path: &str, output_path: &str) -> io::Result<()> {
        let data = fs::read(input_path)?;
        let nonce = Nonce::from_slice(&[0u8; 12]);
        
        let encrypted_data = self.cipher
            .encrypt(nonce, data.as_ref())
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        
        fs::write(output_path, encrypted_data)
    }

    pub fn decrypt_file(&self, input_path: &str, output_path: &str) -> io::Result<()> {
        let encrypted_data = fs::read(input_path)?;
        let nonce = Nonce::from_slice(&[0u8; 12]);
        
        let decrypted_data = self.cipher
            .decrypt(nonce, encrypted_data.as_ref())
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        
        fs::write(output_path, decrypted_data)
    }
}

pub fn create_sample_file(path: &str, content: &[u8]) -> io::Result<()> {
    let mut file = fs::File::create(path)?;
    file.write_all(content)
}