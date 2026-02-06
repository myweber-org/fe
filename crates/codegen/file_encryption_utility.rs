use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce as ChaChaNonce};
use std::error::Error;

pub struct EncryptionResult {
    pub ciphertext: Vec<u8>,
    pub nonce: Vec<u8>,
}

pub enum EncryptionAlgorithm {
    Aes256Gcm,
    ChaCha20Poly1305,
}

pub fn encrypt_data(
    plaintext: &[u8],
    algorithm: EncryptionAlgorithm,
) -> Result<EncryptionResult, Box<dyn Error>> {
    match algorithm {
        EncryptionAlgorithm::Aes256Gcm => {
            let key = Aes256Gcm::generate_key(&mut OsRng);
            let cipher = Aes256Gcm::new(&key);
            let nonce = Nonce::from_slice(&[0u8; 12]);
            let ciphertext = cipher.encrypt(nonce, plaintext)?;
            
            Ok(EncryptionResult {
                ciphertext,
                nonce: nonce.to_vec(),
            })
        }
        EncryptionAlgorithm::ChaCha20Poly1305 => {
            let key = ChaCha20Poly1305::generate_key(&mut OsRng);
            let cipher = ChaCha20Poly1305::new(&key);
            let nonce = ChaChaNonce::from_slice(&[0u8; 12]);
            let ciphertext = cipher.encrypt(nonce, plaintext)?;
            
            Ok(EncryptionResult {
                ciphertext,
                nonce: nonce.to_vec(),
            })
        }
    }
}

pub fn decrypt_data(
    ciphertext: &[u8],
    nonce: &[u8],
    algorithm: EncryptionAlgorithm,
) -> Result<Vec<u8>, Box<dyn Error>> {
    match algorithm {
        EncryptionAlgorithm::Aes256Gcm => {
            let key = Aes256Gcm::generate_key(&mut OsRng);
            let cipher = Aes256Gcm::new(&key);
            let nonce = Nonce::from_slice(nonce);
            let plaintext = cipher.decrypt(nonce, ciphertext)?;
            Ok(plaintext)
        }
        EncryptionAlgorithm::ChaCha20Poly1305 => {
            let key = ChaCha20Poly1305::generate_key(&mut OsRng);
            let cipher = ChaCha20Poly1305::new(&key);
            let nonce = ChaChaNonce::from_slice(nonce);
            let plaintext = cipher.decrypt(nonce, ciphertext)?;
            Ok(plaintext)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aes_encryption_decryption() {
        let plaintext = b"Test encryption data";
        let result = encrypt_data(plaintext, EncryptionAlgorithm::Aes256Gcm).unwrap();
        let decrypted = decrypt_data(&result.ciphertext, &result.nonce, EncryptionAlgorithm::Aes256Gcm).unwrap();
        assert_eq!(plaintext.to_vec(), decrypted);
    }

    #[test]
    fn test_chacha_encryption_decryption() {
        let plaintext = b"Another test message";
        let result = encrypt_data(plaintext, EncryptionAlgorithm::ChaCha20Poly1305).unwrap();
        let decrypted = decrypt_data(&result.ciphertext, &result.nonce, EncryptionAlgorithm::ChaCha20Poly1305).unwrap();
        assert_eq!(plaintext.to_vec(), decrypted);
    }
}use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

pub struct XORCipher {
    key: Vec<u8>,
}

impl XORCipher {
    pub fn new(key: &str) -> Self {
        XORCipher {
            key: key.as_bytes().to_vec(),
        }
    }

    pub fn encrypt_file(&self, source_path: &Path, dest_path: &Path) -> io::Result<()> {
        self.process_file(source_path, dest_path)
    }

    pub fn decrypt_file(&self, source_path: &Path, dest_path: &Path) -> io::Result<()> {
        self.process_file(source_path, dest_path)
    }

    fn process_file(&self, source_path: &Path, dest_path: &Path) -> io::Result<()> {
        let mut source_file = fs::File::open(source_path)?;
        let mut dest_file = fs::File::create(dest_path)?;

        let mut buffer = [0; 4096];
        let mut key_index = 0;

        loop {
            let bytes_read = source_file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }

            let mut processed_buffer = buffer[..bytes_read].to_vec();
            self.xor_transform(&mut processed_buffer, &mut key_index);

            dest_file.write_all(&processed_buffer)?;
        }

        dest_file.flush()?;
        Ok(())
    }

    fn xor_transform(&self, data: &mut [u8], key_index: &mut usize) {
        for byte in data.iter_mut() {
            *byte ^= self.key[*key_index];
            *key_index = (*key_index + 1) % self.key.len();
        }
    }
}

pub fn validate_key(key: &str) -> Result<(), String> {
    if key.is_empty() {
        return Err("Encryption key cannot be empty".to_string());
    }
    if key.len() < 8 {
        return Err("Encryption key should be at least 8 characters".to_string());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_xor_cipher_symmetry() {
        let cipher = XORCipher::new("strong_encryption_key_123!");
        let test_data = b"Hello, this is a secret message!";
        
        let mut encrypted = test_data.to_vec();
        let mut key_index = 0;
        cipher.xor_transform(&mut encrypted, &mut key_index);
        
        key_index = 0;
        cipher.xor_transform(&mut encrypted, &mut key_index);
        
        assert_eq!(encrypted, test_data);
    }

    #[test]
    fn test_file_encryption_decryption() {
        let cipher = XORCipher::new("test_key_890");
        let source_content = b"Sample file content for encryption test";
        
        let source_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(source_file.path(), source_content).unwrap();
        
        cipher.encrypt_file(source_file.path(), encrypted_file.path()).unwrap();
        cipher.decrypt_file(encrypted_file.path(), decrypted_file.path()).unwrap();
        
        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(decrypted_content, source_content);
    }

    #[test]
    fn test_key_validation() {
        assert!(validate_key("").is_err());
        assert!(validate_key("short").is_err());
        assert!(validate_key("valid_key_123").is_ok());
    }
}