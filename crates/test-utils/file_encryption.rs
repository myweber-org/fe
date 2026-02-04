
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use std::error::Error;

pub fn encrypt_data(plaintext: &[u8]) -> Result<(Vec<u8>, Vec<u8>), Box<dyn Error>> {
    let key = Aes256Gcm::generate_key(&mut OsRng);
    let cipher = Aes256Gcm::new(&key);
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    
    let ciphertext = cipher.encrypt(&nonce, plaintext)?;
    Ok((ciphertext, nonce.to_vec()))
}

pub fn decrypt_data(ciphertext: &[u8], nonce: &[u8], key: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let nonce = Nonce::from_slice(nonce);
    
    let plaintext = cipher.decrypt(nonce, ciphertext)?;
    Ok(plaintext)
}

pub fn generate_key() -> Vec<u8> {
    Aes256Gcm::generate_key(&mut OsRng).to_vec()
}
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

pub struct XorCipher {
    key: Vec<u8>,
    key_index: usize,
}

impl XorCipher {
    pub fn new(key: &str) -> Self {
        XorCipher {
            key: key.as_bytes().to_vec(),
            key_index: 0,
        }
    }

    pub fn encrypt_file(&mut self, source_path: &Path, dest_path: &Path) -> io::Result<()> {
        self.process_file(source_path, dest_path)
    }

    pub fn decrypt_file(&mut self, source_path: &Path, dest_path: &Path) -> io::Result<()> {
        self.process_file(source_path, dest_path)
    }

    fn process_file(&mut self, source_path: &Path, dest_path: &Path) -> io::Result<()> {
        let mut source_file = fs::File::open(source_path)?;
        let mut dest_file = fs::File::create(dest_path)?;

        let mut buffer = [0; 4096];
        self.key_index = 0;

        loop {
            let bytes_read = source_file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }

            let processed_chunk = self.process_chunk(&buffer[..bytes_read]);
            dest_file.write_all(&processed_chunk)?;
        }

        dest_file.flush()?;
        Ok(())
    }

    fn process_chunk(&mut self, data: &[u8]) -> Vec<u8> {
        data.iter()
            .map(|&byte| {
                let key_byte = self.key[self.key_index];
                self.key_index = (self.key_index + 1) % self.key.len();
                byte ^ key_byte
            })
            .collect()
    }
}

pub fn validate_key(key: &str) -> Result<(), String> {
    if key.is_empty() {
        return Err("Encryption key cannot be empty".to_string());
    }
    if key.len() < 8 {
        return Err("Encryption key must be at least 8 characters".to_string());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_xor_cipher_symmetry() {
        let original_text = b"Hello, this is a secret message!";
        let key = "my_strong_password_123";
        let mut cipher = XorCipher::new(key);

        let encrypted: Vec<u8> = original_text
            .iter()
            .enumerate()
            .map(|(i, &byte)| {
                let key_byte = key.as_bytes()[i % key.len()];
                byte ^ key_byte
            })
            .collect();

        cipher.key_index = 0;
        let decrypted: Vec<u8> = encrypted
            .iter()
            .map(|&byte| {
                let key_byte = key.as_bytes()[cipher.key_index % key.len()];
                cipher.key_index += 1;
                byte ^ key_byte
            })
            .collect();

        assert_eq!(original_text.to_vec(), decrypted);
    }

    #[test]
    fn test_file_encryption() -> io::Result<()> {
        let test_content = b"Test data for encryption demonstration";
        let key = "secure_key_987";

        let mut source_file = NamedTempFile::new()?;
        source_file.write_all(test_content)?;

        let encrypted_file = NamedTempFile::new()?;
        let decrypted_file = NamedTempFile::new()?;

        let mut cipher = XorCipher::new(key);
        cipher.encrypt_file(source_file.path(), encrypted_file.path())?;

        cipher.key_index = 0;
        cipher.decrypt_file(encrypted_file.path(), decrypted_file.path())?;

        let mut decrypted_content = Vec::new();
        fs::File::open(decrypted_file.path())?.read_to_end(&mut decrypted_content)?;

        assert_eq!(test_content.to_vec(), decrypted_content);
        Ok(())
    }

    #[test]
    fn test_key_validation() {
        assert!(validate_key("").is_err());
        assert!(validate_key("short").is_err());
        assert!(validate_key("valid_password").is_ok());
    }
}