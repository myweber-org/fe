
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use std::error::Error;

pub struct FileEncryptor {
    cipher: Aes256Gcm,
}

impl FileEncryptor {
    pub fn new() -> Self {
        let key = Aes256Gcm::generate_key(&mut OsRng);
        let cipher = Aes256Gcm::new(&key);
        Self { cipher }
    }

    pub fn encrypt_data(&self, plaintext: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let ciphertext = self
            .cipher
            .encrypt(&nonce, plaintext)
            .map_err(|e| format!("Encryption failed: {}", e))?;

        let mut result = Vec::with_capacity(nonce.len() + ciphertext.len());
        result.extend_from_slice(nonce.as_slice());
        result.extend_from_slice(&ciphertext);
        Ok(result)
    }

    pub fn decrypt_data(&self, ciphertext: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
        if ciphertext.len() < 12 {
            return Err("Invalid ciphertext length".into());
        }

        let nonce = Nonce::from_slice(&ciphertext[..12]);
        let encrypted_data = &ciphertext[12..];

        self.cipher
            .decrypt(nonce, encrypted_data)
            .map_err(|e| format!("Decryption failed: {}", e).into())
    }
}

pub fn process_encryption() -> Result<(), Box<dyn Error>> {
    let encryptor = FileEncryptor::new();
    let test_data = b"Confidential information requiring secure storage";

    let encrypted = encryptor.encrypt_data(test_data)?;
    println!("Encrypted data length: {} bytes", encrypted.len());

    let decrypted = encryptor.decrypt_data(&encrypted)?;
    assert_eq!(test_data.to_vec(), decrypted);
    println!("Decryption successful, data integrity verified");

    Ok(())
}use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use chacha20poly1305::{ChaCha20Poly1305, Key as ChaChaKey, Nonce as ChaChaNonce};
use rand::RngCore;
use std::fs;
use std::path::Path;

#[derive(Debug)]
pub enum EncryptionError {
    IoError(std::io::Error),
    CryptoError(String),
}

impl From<std::io::Error> for EncryptionError {
    fn from(err: std::io::Error) -> Self {
        EncryptionError::IoError(err)
    }
}

pub struct FileEncryptor {
    algorithm: EncryptionAlgorithm,
}

#[derive(Debug, Clone, Copy)]
pub enum EncryptionAlgorithm {
    Aes256Gcm,
    ChaCha20Poly1305,
}

impl FileEncryptor {
    pub fn new(algorithm: EncryptionAlgorithm) -> Self {
        FileEncryptor { algorithm }
    }

    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<Vec<u8>, EncryptionError> {
        let data = fs::read(input_path)?;
        
        match self.algorithm {
            EncryptionAlgorithm::Aes256Gcm => self.aes_encrypt(&data),
            EncryptionAlgorithm::ChaCha20Poly1305 => self.chacha_encrypt(&data),
        }?;
        
        fs::write(output_path, &data)?;
        Ok(data)
    }

    fn aes_encrypt(&self, data: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        let key = Key::<Aes256Gcm>::generate(&mut OsRng);
        let cipher = Aes256Gcm::new(&key);
        
        let mut nonce = [0u8; 12];
        OsRng.fill_bytes(&mut nonce);
        let nonce = Nonce::from_slice(&nonce);
        
        cipher.encrypt(nonce, data)
            .map_err(|e| EncryptionError::CryptoError(e.to_string()))
    }

    fn chacha_encrypt(&self, data: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        let key = ChaChaKey::generate(&mut OsRng);
        let cipher = ChaCha20Poly1305::new(&key);
        
        let mut nonce = [0u8; 12];
        OsRng.fill_bytes(&mut nonce);
        let nonce = ChaChaNonce::from_slice(&nonce);
        
        cipher.encrypt(nonce, data)
            .map_err(|e| EncryptionError::CryptoError(e.to_string()))
    }

    pub fn decrypt_file(&self, encrypted_data: &[u8], algorithm: EncryptionAlgorithm) -> Result<Vec<u8>, EncryptionError> {
        match algorithm {
            EncryptionAlgorithm::Aes256Gcm => self.aes_decrypt(encrypted_data),
            EncryptionAlgorithm::ChaCha20Poly1305 => self.chacha_decrypt(encrypted_data),
        }
    }

    fn aes_decrypt(&self, data: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        if data.len() < 12 {
            return Err(EncryptionError::CryptoError("Invalid encrypted data".to_string()));
        }
        
        let (nonce_bytes, ciphertext) = data.split_at(12);
        let key = Key::<Aes256Gcm>::generate(&mut OsRng);
        let cipher = Aes256Gcm::new(&key);
        let nonce = Nonce::from_slice(nonce_bytes);
        
        cipher.decrypt(nonce, ciphertext)
            .map_err(|e| EncryptionError::CryptoError(e.to_string()))
    }

    fn chacha_decrypt(&self, data: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        if data.len() < 12 {
            return Err(EncryptionError::CryptoError("Invalid encrypted data".to_string()));
        }
        
        let (nonce_bytes, ciphertext) = data.split_at(12);
        let key = ChaChaKey::generate(&mut OsRng);
        let cipher = ChaCha20Poly1305::new(&key);
        let nonce = ChaChaNonce::from_slice(nonce_bytes);
        
        cipher.decrypt(nonce, ciphertext)
            .map_err(|e| EncryptionError::CryptoError(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_aes_encryption_roundtrip() {
        let encryptor = FileEncryptor::new(EncryptionAlgorithm::Aes256Gcm);
        let test_data = b"Test encryption data";
        
        let encrypted = encryptor.aes_encrypt(test_data).unwrap();
        let decrypted = encryptor.aes_decrypt(&encrypted).unwrap();
        
        assert_eq!(test_data, decrypted.as_slice());
    }

    #[test]
    fn test_chacha_encryption_roundtrip() {
        let encryptor = FileEncryptor::new(EncryptionAlgorithm::ChaCha20Poly1305);
        let test_data = b"Test encryption data";
        
        let encrypted = encryptor.chacha_encrypt(test_data).unwrap();
        let decrypted = encryptor.chacha_decrypt(&encrypted).unwrap();
        
        assert_eq!(test_data, decrypted.as_slice());
    }
}use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use chacha20poly1305::{ChaCha20Poly1305, Key as ChaChaKey, Nonce as ChaChaNonce};
use std::fs;
use std::io::{self, Write};
use std::path::Path;

pub struct EncryptionResult {
    pub ciphertext: Vec<u8>,
    pub nonce: Vec<u8>,
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
        Self { algorithm }
    }

    pub fn generate_key(&self) -> Vec<u8> {
        match self.algorithm {
            Algorithm::Aes256Gcm => {
                let key = Aes256Gcm::generate_key(&mut OsRng);
                key.as_slice().to_vec()
            }
            Algorithm::ChaCha20Poly1305 => {
                let key = ChaCha20Poly1305::generate_key(&mut OsRng);
                key.as_slice().to_vec()
            }
        }
    }

    pub fn encrypt_file(&self, input_path: &Path, key: &[u8]) -> io::Result<EncryptionResult> {
        let plaintext = fs::read(input_path)?;
        
        match self.algorithm {
            Algorithm::Aes256Gcm => {
                let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
                let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
                let ciphertext = cipher
                    .encrypt(&nonce, plaintext.as_ref())
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
                
                Ok(EncryptionResult {
                    ciphertext,
                    nonce: nonce.to_vec(),
                })
            }
            Algorithm::ChaCha20Poly1305 => {
                let cipher = ChaCha20Poly1305::new(ChaChaKey::from_slice(key));
                let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);
                let ciphertext = cipher
                    .encrypt(&nonce, plaintext.as_ref())
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
                
                Ok(EncryptionResult {
                    ciphertext,
                    nonce: nonce.to_vec(),
                })
            }
        }
    }

    pub fn decrypt_file(&self, ciphertext: &[u8], key: &[u8], nonce: &[u8]) -> io::Result<Vec<u8>> {
        match self.algorithm {
            Algorithm::Aes256Gcm => {
                let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
                let nonce = Nonce::from_slice(nonce);
                cipher
                    .decrypt(nonce, ciphertext)
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
            }
            Algorithm::ChaCha20Poly1305 => {
                let cipher = ChaCha20Poly1305::new(ChaChaKey::from_slice(key));
                let nonce = ChaChaNonce::from_slice(nonce);
                cipher
                    .decrypt(nonce, ciphertext)
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
            }
        }
    }

    pub fn save_encrypted(&self, result: &EncryptionResult, output_path: &Path) -> io::Result<()> {
        let mut file = fs::File::create(output_path)?;
        file.write_all(&result.nonce)?;
        file.write_all(&result.ciphertext)?;
        Ok(())
    }

    pub fn load_encrypted(&self, input_path: &Path) -> io::Result<(Vec<u8>, Vec<u8>)> {
        let data = fs::read(input_path)?;
        
        let nonce_size = match self.algorithm {
            Algorithm::Aes256Gcm => 12,
            Algorithm::ChaCha20Poly1305 => 12,
        };
        
        if data.len() < nonce_size {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "File too small to contain nonce",
            ));
        }
        
        let nonce = data[..nonce_size].to_vec();
        let ciphertext = data[nonce_size..].to_vec();
        
        Ok((ciphertext, nonce))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_aes_encryption_decryption() {
        let encryptor = FileEncryptor::new(Algorithm::Aes256Gcm);
        let key = encryptor.generate_key();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "Test data for encryption").unwrap();
        
        let result = encryptor
            .encrypt_file(temp_file.path(), &key)
            .unwrap();
        
        let decrypted = encryptor
            .decrypt_file(&result.ciphertext, &key, &result.nonce)
            .unwrap();
        
        assert_eq!(decrypted, b"Test data for encryption\n");
    }

    #[test]
    fn test_chacha_encryption_decryption() {
        let encryptor = FileEncryptor::new(Algorithm::ChaCha20Poly1305);
        let key = encryptor.generate_key();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "ChaCha test data").unwrap();
        
        let result = encryptor
            .encrypt_file(temp_file.path(), &key)
            .unwrap();
        
        let decrypted = encryptor
            .decrypt_file(&result.ciphertext, &key, &result.nonce)
            .unwrap();
        
        assert_eq!(decrypted, b"ChaCha test data\n");
    }
}