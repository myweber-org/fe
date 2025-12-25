use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use chacha20poly1305::{ChaCha20Poly1305, Key as ChaChaKey, Nonce as ChaChaNonce};
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

#[derive(Debug)]
pub enum EncryptionError {
    IoError(io::Error),
    CryptoError(String),
}

impl From<io::Error> for EncryptionError {
    fn from(err: io::Error) -> Self {
        EncryptionError::IoError(err)
    }
}

pub enum Algorithm {
    Aes256Gcm,
    ChaCha20Poly1305,
}

pub struct FileEncryptor {
    algorithm: Algorithm,
}

impl FileEncryptor {
    pub fn new(algorithm: Algorithm) -> Self {
        FileEncryptor { algorithm }
    }

    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path, key: &[u8]) -> Result<(), EncryptionError> {
        let mut input_file = fs::File::open(input_path)?;
        let mut plaintext = Vec::new();
        input_file.read_to_end(&mut plaintext)?;

        let (ciphertext, nonce) = match self.algorithm {
            Algorithm::Aes256Gcm => self.encrypt_aes(&plaintext, key)?,
            Algorithm::ChaCha20Poly1305 => self.encrypt_chacha(&plaintext, key)?,
        };

        let mut output_file = fs::File::create(output_path)?;
        output_file.write_all(&nonce)?;
        output_file.write_all(&ciphertext)?;

        Ok(())
    }

    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path, key: &[u8]) -> Result<(), EncryptionError> {
        let mut input_file = fs::File::open(input_path)?;
        let mut encrypted_data = Vec::new();
        input_file.read_to_end(&mut encrypted_data)?;

        let (nonce, ciphertext) = match self.algorithm {
            Algorithm::Aes256Gcm => self.split_aes_data(&encrypted_data)?,
            Algorithm::ChaCha20Poly1305 => self.split_chacha_data(&encrypted_data)?,
        };

        let plaintext = match self.algorithm {
            Algorithm::Aes256Gcm => self.decrypt_aes(&ciphertext, key, &nonce)?,
            Algorithm::ChaCha20Poly1305 => self.decrypt_chacha(&ciphertext, key, &nonce)?,
        };

        let mut output_file = fs::File::create(output_path)?;
        output_file.write_all(&plaintext)?;

        Ok(())
    }

    fn encrypt_aes(&self, plaintext: &[u8], key: &[u8]) -> Result<(Vec<u8>, Vec<u8>), EncryptionError> {
        let key = Key::<Aes256Gcm>::from_slice(key);
        let cipher = Aes256Gcm::new(key);
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        
        cipher.encrypt(&nonce, plaintext)
            .map(|ciphertext| (ciphertext, nonce.to_vec()))
            .map_err(|e| EncryptionError::CryptoError(e.to_string()))
    }

    fn decrypt_aes(&self, ciphertext: &[u8], key: &[u8], nonce: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        let key = Key::<Aes256Gcm>::from_slice(key);
        let cipher = Aes256Gcm::new(key);
        let nonce = Nonce::from_slice(nonce);
        
        cipher.decrypt(nonce, ciphertext)
            .map_err(|e| EncryptionError::CryptoError(e.to_string()))
    }

    fn encrypt_chacha(&self, plaintext: &[u8], key: &[u8]) -> Result<(Vec<u8>, Vec<u8>), EncryptionError> {
        let key = ChaChaKey::from_slice(key);
        let cipher = ChaCha20Poly1305::new(key);
        let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);
        
        cipher.encrypt(&nonce, plaintext)
            .map(|ciphertext| (ciphertext, nonce.to_vec()))
            .map_err(|e| EncryptionError::CryptoError(e.to_string()))
    }

    fn decrypt_chacha(&self, ciphertext: &[u8], key: &[u8], nonce: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        let key = ChaChaKey::from_slice(key);
        let cipher = ChaCha20Poly1305::new(key);
        let nonce = ChaChaNonce::from_slice(nonce);
        
        cipher.decrypt(nonce, ciphertext)
            .map_err(|e| EncryptionError::CryptoError(e.to_string()))
    }

    fn split_aes_data(&self, data: &[u8]) -> Result<(Vec<u8>, Vec<u8>), EncryptionError> {
        if data.len() < 12 {
            return Err(EncryptionError::CryptoError("Invalid encrypted data".to_string()));
        }
        let (nonce, ciphertext) = data.split_at(12);
        Ok((nonce.to_vec(), ciphertext.to_vec()))
    }

    fn split_chacha_data(&self, data: &[u8]) -> Result<(Vec<u8>, Vec<u8>), EncryptionError> {
        if data.len() < 12 {
            return Err(EncryptionError::CryptoError("Invalid encrypted data".to_string()));
        }
        let (nonce, ciphertext) = data.split_at(12);
        Ok((nonce.to_vec(), ciphertext.to_vec()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_aes_encryption_decryption() {
        let key = [0u8; 32];
        let plaintext = b"Test secret message";
        
        let encryptor = FileEncryptor::new(Algorithm::Aes256Gcm);
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), plaintext).unwrap();
        
        encryptor.encrypt_file(input_file.path(), encrypted_file.path(), &key).unwrap();
        encryptor.decrypt_file(encrypted_file.path(), decrypted_file.path(), &key).unwrap();
        
        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(decrypted_content, plaintext);
    }

    #[test]
    fn test_chacha_encryption_decryption() {
        let key = [0u8; 32];
        let plaintext = b"Another test message";
        
        let encryptor = FileEncryptor::new(Algorithm::ChaCha20Poly1305);
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), plaintext).unwrap();
        
        encryptor.encrypt_file(input_file.path(), encrypted_file.path(), &key).unwrap();
        encryptor.decrypt_file(encrypted_file.path(), decrypted_file.path(), &key).unwrap();
        
        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(decrypted_content, plaintext);
    }
}