use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce as ChaChaNonce};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EncryptionAlgorithm {
    Aes256Gcm,
    ChaCha20Poly1305,
}

pub struct FileEncryptor {
    algorithm: EncryptionAlgorithm,
}

impl FileEncryptor {
    pub fn new(algorithm: EncryptionAlgorithm) -> Self {
        Self { algorithm }
    }

    pub fn encrypt_file(
        &self,
        input_path: &Path,
        output_path: &Path,
        key: &[u8],
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut input_file = File::open(input_path)?;
        let mut plaintext = Vec::new();
        input_file.read_to_end(&mut plaintext)?;

        let ciphertext = match self.algorithm {
            EncryptionAlgorithm::Aes256Gcm => self.encrypt_aes_gcm(&plaintext, key)?,
            EncryptionAlgorithm::ChaCha20Poly1305 => self.encrypt_chacha(&plaintext, key)?,
        };

        let mut output_file = File::create(output_path)?;
        output_file.write_all(&ciphertext)?;

        Ok(())
    }

    pub fn decrypt_file(
        &self,
        input_path: &Path,
        output_path: &Path,
        key: &[u8],
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut input_file = File::open(input_path)?;
        let mut ciphertext = Vec::new();
        input_file.read_to_end(&mut ciphertext)?;

        let plaintext = match self.algorithm {
            EncryptionAlgorithm::Aes256Gcm => self.decrypt_aes_gcm(&ciphertext, key)?,
            EncryptionAlgorithm::ChaCha20Poly1305 => self.decrypt_chacha(&ciphertext, key)?,
        };

        let mut output_file = File::create(output_path)?;
        output_file.write_all(&plaintext)?;

        Ok(())
    }

    fn encrypt_aes_gcm(&self, plaintext: &[u8], key: &[u8]) -> Result<Vec<u8>, aes_gcm::Error> {
        let cipher = Aes256Gcm::new_from_slice(key).unwrap();
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let ciphertext = cipher.encrypt(&nonce, plaintext)?;
        
        let mut result = Vec::with_capacity(nonce.len() + ciphertext.len());
        result.extend_from_slice(nonce.as_slice());
        result.extend_from_slice(&ciphertext);
        
        Ok(result)
    }

    fn decrypt_aes_gcm(&self, ciphertext: &[u8], key: &[u8]) -> Result<Vec<u8>, aes_gcm::Error> {
        let cipher = Aes256Gcm::new_from_slice(key).unwrap();
        
        if ciphertext.len() < 12 {
            return Err(aes_gcm::Error);
        }
        
        let nonce = Nonce::from_slice(&ciphertext[..12]);
        let encrypted_data = &ciphertext[12..];
        
        cipher.decrypt(nonce, encrypted_data)
    }

    fn encrypt_chacha(&self, plaintext: &[u8], key: &[u8]) -> Result<Vec<u8>, chacha20poly1305::Error> {
        let cipher = ChaCha20Poly1305::new(Key::from_slice(key));
        let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);
        let ciphertext = cipher.encrypt(&nonce, plaintext)?;
        
        let mut result = Vec::with_capacity(nonce.len() + ciphertext.len());
        result.extend_from_slice(nonce.as_slice());
        result.extend_from_slice(&ciphertext);
        
        Ok(result)
    }

    fn decrypt_chacha(&self, ciphertext: &[u8], key: &[u8]) -> Result<Vec<u8>, chacha20poly1305::Error> {
        let cipher = ChaCha20Poly1305::new(Key::from_slice(key));
        
        if ciphertext.len() < 12 {
            return Err(chacha20poly1305::Error);
        }
        
        let nonce = ChaChaNonce::from_slice(&ciphertext[..12]);
        let encrypted_data = &ciphertext[12..];
        
        cipher.decrypt(nonce, encrypted_data)
    }
}

pub fn generate_random_key() -> [u8; 32] {
    let mut key = [0u8; 32];
    OsRng.fill_bytes(&mut key);
    key
}

pub fn secure_delete_file(path: &Path) -> Result<(), std::io::Error> {
    let file_size = fs::metadata(path)?.len();
    
    for _ in 0..3 {
        let random_data: Vec<u8> = (0..file_size)
            .map(|_| rand::random::<u8>())
            .collect();
        
        fs::write(path, &random_data)?;
    }
    
    fs::remove_file(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_aes_encryption_decryption() {
        let encryptor = FileEncryptor::new(EncryptionAlgorithm::Aes256Gcm);
        let key = generate_random_key();
        
        let plaintext = b"Test data for encryption";
        
        let ciphertext = encryptor.encrypt_aes_gcm(plaintext, &key).unwrap();
        let decrypted = encryptor.decrypt_aes_gcm(&ciphertext, &key).unwrap();
        
        assert_eq!(plaintext.to_vec(), decrypted);
    }

    #[test]
    fn test_chacha_encryption_decryption() {
        let encryptor = FileEncryptor::new(EncryptionAlgorithm::ChaCha20Poly1305);
        let key = generate_random_key();
        
        let plaintext = b"Test data for ChaCha encryption";
        
        let ciphertext = encryptor.encrypt_chacha(plaintext, &key).unwrap();
        let decrypted = encryptor.decrypt_chacha(&ciphertext, &key).unwrap();
        
        assert_eq!(plaintext.to_vec(), decrypted);
    }

    #[test]
    fn test_file_encryption() {
        let encryptor = FileEncryptor::new(EncryptionAlgorithm::Aes256Gcm);
        let key = generate_random_key();
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        let test_data = b"File encryption test data";
        fs::write(input_file.path(), test_data).unwrap();
        
        encryptor.encrypt_file(input_file.path(), encrypted_file.path(), &key).unwrap();
        encryptor.decrypt_file(encrypted_file.path(), decrypted_file.path(), &key).unwrap();
        
        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(test_data.to_vec(), decrypted_data);
    }
}