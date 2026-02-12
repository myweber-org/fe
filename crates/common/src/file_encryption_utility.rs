use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};
use std::{
    fs::{self, File},
    io::{Read, Write},
    path::Path,
};

const SALT_LENGTH: usize = 16;
const NONCE_LENGTH: usize = 12;

pub struct FileEncryptor {
    cipher: Aes256Gcm,
}

impl FileEncryptor {
    pub fn new(password: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2.hash_password(password.as_bytes(), &salt)?;
        let key = Key::<Aes256Gcm>::from_slice(password_hash.hash.unwrap().as_bytes());
        Ok(Self {
            cipher: Aes256Gcm::new(key),
        })
    }

    pub fn encrypt_file(
        &self,
        input_path: &Path,
        output_path: &Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut file = File::open(input_path)?;
        let mut plaintext = Vec::new();
        file.read_to_end(&mut plaintext)?;

        let nonce = Nonce::generate(&mut OsRng);
        let ciphertext = self.cipher.encrypt(&nonce, plaintext.as_ref())?;

        let mut output = File::create(output_path)?;
        output.write_all(&nonce)?;
        output.write_all(&ciphertext)?;

        Ok(())
    }

    pub fn decrypt_file(
        &self,
        input_path: &Path,
        output_path: &Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut file = File::open(input_path)?;
        let mut encrypted_data = Vec::new();
        file.read_to_end(&mut encrypted_data)?;

        if encrypted_data.len() < NONCE_LENGTH {
            return Err("Invalid encrypted file format".into());
        }

        let (nonce_bytes, ciphertext) = encrypted_data.split_at(NONCE_LENGTH);
        let nonce = Nonce::from_slice(nonce_bytes);
        let plaintext = self.cipher.decrypt(nonce, ciphertext)?;

        let mut output = File::create(output_path)?;
        output.write_all(&plaintext)?;

        Ok(())
    }
}

pub fn generate_random_key() -> [u8; 32] {
    let mut key = [0u8; 32];
    OsRng.fill_bytes(&mut key);
    key
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let password = "secure_password_123";
        let encryptor = FileEncryptor::new(password).unwrap();

        let test_data = b"Hello, this is a secret message!";
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        fs::write(input_file.path(), test_data).unwrap();

        encryptor
            .encrypt_file(input_file.path(), encrypted_file.path())
            .unwrap();
        encryptor
            .decrypt_file(encrypted_file.path(), decrypted_file.path())
            .unwrap();

        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(test_data.to_vec(), decrypted_data);
    }

    #[test]
    fn test_wrong_password_fails() {
        let password1 = "password_one";
        let password2 = "password_two";

        let encryptor1 = FileEncryptor::new(password1).unwrap();
        let encryptor2 = FileEncryptor::new(password2).unwrap();

        let test_data = b"Test data";
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();

        fs::write(input_file.path(), test_data).unwrap();
        encryptor1
            .encrypt_file(input_file.path(), encrypted_file.path())
            .unwrap();

        let result = encryptor2.decrypt_file(encrypted_file.path(), NamedTempFile::new().unwrap().path());
        assert!(result.is_err());
    }
}
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

pub struct FileEncryptor {
    cipher: Aes256Gcm,
}

impl FileEncryptor {
    pub fn from_password(password: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        
        let password_hash = argon2.hash_password(password.as_bytes(), &salt)?;
        let key_bytes = password_hash.hash.unwrap().as_bytes();
        
        let key = Key::<Aes256Gcm>::from_slice(&key_bytes[..32]);
        let cipher = Aes256Gcm::new(key);
        
        Ok(FileEncryptor { cipher })
    }
    
    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let mut file_data = Vec::new();
        let mut file = fs::File::open(input_path)?;
        file.read_to_end(&mut file_data)?;
        
        let nonce_bytes: [u8; NONCE_SIZE] = OsRng.fill_bytes();
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        let ciphertext = self.cipher.encrypt(nonce, file_data.as_ref())
            .map_err(|e| format!("Encryption failed: {}", e))?;
        
        let mut output_file = fs::File::create(output_path)?;
        output_file.write_all(&nonce_bytes)?;
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
        
        let plaintext = self.cipher.decrypt(nonce, ciphertext)
            .map_err(|e| format!("Decryption failed: {}", e))?;
        
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
        let test_data = b"Secret data that needs protection";
        let password = "strong_password_123!";
        
        let encryptor = FileEncryptor::from_password(password).unwrap();
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), test_data).unwrap();
        
        encryptor.encrypt_file(input_file.path(), encrypted_file.path()).unwrap();
        encryptor.decrypt_file(encrypted_file.path(), decrypted_file.path()).unwrap();
        
        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(test_data.to_vec(), decrypted_data);
    }
}use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce as ChaChaNonce};
use rand::{rngs::OsRng, RngCore};
use std::fs;
use std::io::{Read, Write};
use std::path::Path;

const AES_NONCE_SIZE: usize = 12;
const CHACHA_NONCE_SIZE: usize = 12;
const KEY_SIZE: usize = 32;

#[derive(Debug)]
pub enum EncryptionError {
    IoError(std::io::Error),
    CryptoError(String),
    InvalidKeySize,
    InvalidNonceSize,
}

impl From<std::io::Error> for EncryptionError {
    fn from(err: std::io::Error) -> Self {
        EncryptionError::IoError(err)
    }
}

pub struct EncryptionResult {
    pub ciphertext: Vec<u8>,
    pub nonce: Vec<u8>,
}

pub trait Encryptor {
    fn encrypt(&self, plaintext: &[u8], key: &[u8]) -> Result<EncryptionResult, EncryptionError>;
    fn decrypt(&self, ciphertext: &[u8], key: &[u8], nonce: &[u8]) -> Result<Vec<u8>, EncryptionError>;
}

pub struct Aes256GcmEncryptor;

impl Encryptor for Aes256GcmEncryptor {
    fn encrypt(&self, plaintext: &[u8], key: &[u8]) -> Result<EncryptionResult, EncryptionError> {
        if key.len() != KEY_SIZE {
            return Err(EncryptionError::InvalidKeySize);
        }

        let cipher = Aes256Gcm::new_from_slice(key)
            .map_err(|e| EncryptionError::CryptoError(e.to_string()))?;

        let mut nonce_bytes = [0u8; AES_NONCE_SIZE];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, plaintext)
            .map_err(|e| EncryptionError::CryptoError(e.to_string()))?;

        Ok(EncryptionResult {
            ciphertext,
            nonce: nonce_bytes.to_vec(),
        })
    }

    fn decrypt(&self, ciphertext: &[u8], key: &[u8], nonce: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        if key.len() != KEY_SIZE {
            return Err(EncryptionError::InvalidKeySize);
        }
        if nonce.len() != AES_NONCE_SIZE {
            return Err(EncryptionError::InvalidNonceSize);
        }

        let cipher = Aes256Gcm::new_from_slice(key)
            .map_err(|e| EncryptionError::CryptoError(e.to_string()))?;

        let nonce = Nonce::from_slice(nonce);
        cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| EncryptionError::CryptoError(e.to_string()))
    }
}

pub struct ChaCha20Poly1305Encryptor;

impl Encryptor for ChaCha20Poly1305Encryptor {
    fn encrypt(&self, plaintext: &[u8], key: &[u8]) -> Result<EncryptionResult, EncryptionError> {
        if key.len() != KEY_SIZE {
            return Err(EncryptionError::InvalidKeySize);
        }

        let key = Key::from_slice(key);
        let cipher = ChaCha20Poly1305::new(key);

        let mut nonce_bytes = [0u8; CHACHA_NONCE_SIZE];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = ChaChaNonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, plaintext)
            .map_err(|e| EncryptionError::CryptoError(e.to_string()))?;

        Ok(EncryptionResult {
            ciphertext,
            nonce: nonce_bytes.to_vec(),
        })
    }

    fn decrypt(&self, ciphertext: &[u8], key: &[u8], nonce: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        if key.len() != KEY_SIZE {
            return Err(EncryptionError::InvalidKeySize);
        }
        if nonce.len() != CHACHA_NONCE_SIZE {
            return Err(EncryptionError::InvalidNonceSize);
        }

        let key = Key::from_slice(key);
        let cipher = ChaCha20Poly1305::new(key);
        let nonce = ChaChaNonce::from_slice(nonce);

        cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| EncryptionError::CryptoError(e.to_string()))
    }
}

pub fn encrypt_file<P: AsRef<Path>>(
    path: P,
    key: &[u8],
    encryptor: &dyn Encryptor,
) -> Result<EncryptionResult, EncryptionError> {
    let mut file = fs::File::open(&path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    encryptor.encrypt(&buffer, key)
}

pub fn decrypt_file<P: AsRef<Path>>(
    path: P,
    key: &[u8],
    nonce: &[u8],
    encryptor: &dyn Encryptor,
) -> Result<Vec<u8>, EncryptionError> {
    let mut file = fs::File::open(&path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    encryptor.decrypt(&buffer, key, nonce)
}

pub fn save_encrypted_data<P: AsRef<Path>>(
    result: &EncryptionResult,
    output_path: P,
) -> Result<(), EncryptionError> {
    let mut file = fs::File::create(output_path)?;
    file.write_all(&result.ciphertext)?;
    Ok(())
}

pub fn generate_random_key() -> Vec<u8> {
    let mut key = vec![0u8; KEY_SIZE];
    OsRng.fill_bytes(&mut key);
    key
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_aes_encryption_decryption() {
        let encryptor = Aes256GcmEncryptor;
        let key = generate_random_key();
        let plaintext = b"Test encryption data";

        let result = encryptor.encrypt(plaintext, &key).unwrap();
        let decrypted = encryptor.decrypt(&result.ciphertext, &key, &result.nonce).unwrap();

        assert_eq!(plaintext.to_vec(), decrypted);
    }

    #[test]
    fn test_chacha_encryption_decryption() {
        let encryptor = ChaCha20Poly1305Encryptor;
        let key = generate_random_key();
        let plaintext = b"Test encryption data";

        let result = encryptor.encrypt(plaintext, &key).unwrap();
        let decrypted = encryptor.decrypt(&result.ciphertext, &key, &result.nonce).unwrap();

        assert_eq!(plaintext.to_vec(), decrypted);
    }

    #[test]
    fn test_file_encryption() {
        let encryptor = Aes256GcmEncryptor;
        let key = generate_random_key();

        let temp_file = NamedTempFile::new().unwrap();
        let test_data = b"File encryption test data";
        fs::write(temp_file.path(), test_data).unwrap();

        let result = encrypt_file(temp_file.path(), &key, &encryptor).unwrap();
        let decrypted = encryptor.decrypt(&result.ciphertext, &key, &result.nonce).unwrap();

        assert_eq!(test_data.to_vec(), decrypted);
    }
}