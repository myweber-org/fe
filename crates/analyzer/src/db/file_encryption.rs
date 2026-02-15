use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use hex;
use rand::RngCore;
use std::fs;
use std::io::{Read, Write};

type Aes256CbcEnc = cbc::Encryptor<aes::Aes256>;
type Aes256CbcDec = cbc::Decryptor<aes::Aes256>;

const KEY_LENGTH: usize = 32;
const IV_LENGTH: usize = 16;

pub fn generate_key() -> Vec<u8> {
    let mut key = vec![0u8; KEY_LENGTH];
    rand::thread_rng().fill_bytes(&mut key);
    key
}

pub fn encrypt_file(input_path: &str, output_path: &str, key: &[u8]) -> Result<(), String> {
    if key.len() != KEY_LENGTH {
        return Err(format!("Key must be {} bytes", KEY_LENGTH));
    }

    let mut file = fs::File::open(input_path)
        .map_err(|e| format!("Failed to open input file: {}", e))?;
    let mut plaintext = Vec::new();
    file.read_to_end(&mut plaintext)
        .map_err(|e| format!("Failed to read input file: {}", e))?;

    let mut iv = [0u8; IV_LENGTH];
    rand::thread_rng().fill_bytes(&mut iv);

    let ciphertext = Aes256CbcEnc::new(key.into(), &iv.into())
        .encrypt_padded_vec_mut::<Pkcs7>(&plaintext);

    let mut output = fs::File::create(output_path)
        .map_err(|e| format!("Failed to create output file: {}", e))?;

    output.write_all(&iv).map_err(|e| format!("Failed to write IV: {}", e))?;
    output.write_all(&ciphertext).map_err(|e| format!("Failed to write ciphertext: {}", e))?;

    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str, key: &[u8]) -> Result<(), String> {
    if key.len() != KEY_LENGTH {
        return Err(format!("Key must be {} bytes", KEY_LENGTH));
    }

    let mut file = fs::File::open(input_path)
        .map_err(|e| format!("Failed to open input file: {}", e))?;
    let mut encrypted_data = Vec::new();
    file.read_to_end(&mut encrypted_data)
        .map_err(|e| format!("Failed to read input file: {}", e))?;

    if encrypted_data.len() < IV_LENGTH {
        return Err("Encrypted data too short".to_string());
    }

    let (iv, ciphertext) = encrypted_data.split_at(IV_LENGTH);
    let plaintext = Aes256CbcDec::new(key.into(), iv.into())
        .decrypt_padded_vec_mut::<Pkcs7>(ciphertext)
        .map_err(|e| format!("Decryption failed: {}", e))?;

    let mut output = fs::File::create(output_path)
        .map_err(|e| format!("Failed to create output file: {}", e))?;
    output.write_all(&plaintext)
        .map_err(|e| format!("Failed to write plaintext: {}", e))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let key = generate_key();
        let test_data = b"Hello, this is a secret message!";

        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        fs::write(input_file.path(), test_data).unwrap();

        encrypt_file(
            input_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap(),
            &key,
        ).unwrap();

        decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap(),
            &key,
        ).unwrap();

        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(test_data.to_vec(), decrypted_data);
    }
}
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use chacha20poly1305::{ChaCha20Poly1305, Key as ChaChaKey, Nonce as ChaChaNonce};
use std::fs;
use std::io::{Read, Write};
use std::path::Path;

#[derive(Debug)]
pub enum EncryptionError {
    IoError(std::io::Error),
    CryptoError(String),
    InvalidKeyLength,
}

impl From<std::io::Error> for EncryptionError {
    fn from(err: std::io::Error) -> Self {
        EncryptionError::IoError(err)
    }
}

pub struct FileEncryptor {
    algorithm: EncryptionAlgorithm,
}

pub enum EncryptionAlgorithm {
    Aes256Gcm,
    ChaCha20Poly1305,
}

impl FileEncryptor {
    pub fn new(algorithm: EncryptionAlgorithm) -> Self {
        FileEncryptor { algorithm }
    }

    pub fn encrypt_file(
        &self,
        input_path: &Path,
        output_path: &Path,
        key: &[u8],
    ) -> Result<(), EncryptionError> {
        let mut file_data = Vec::new();
        let mut file = fs::File::open(input_path)?;
        file.read_to_end(&mut file_data)?;

        let encrypted_data = match self.algorithm {
            EncryptionAlgorithm::Aes256Gcm => self.encrypt_aes(&file_data, key)?,
            EncryptionAlgorithm::ChaCha20Poly1305 => self.encrypt_chacha(&file_data, key)?,
        };

        let mut output_file = fs::File::create(output_path)?;
        output_file.write_all(&encrypted_data)?;

        Ok(())
    }

    pub fn decrypt_file(
        &self,
        input_path: &Path,
        output_path: &Path,
        key: &[u8],
    ) -> Result<(), EncryptionError> {
        let mut encrypted_data = Vec::new();
        let mut file = fs::File::open(input_path)?;
        file.read_to_end(&mut encrypted_data)?;

        let decrypted_data = match self.algorithm {
            EncryptionAlgorithm::Aes256Gcm => self.decrypt_aes(&encrypted_data, key)?,
            EncryptionAlgorithm::ChaCha20Poly1305 => self.decrypt_chacha(&encrypted_data, key)?,
        };

        let mut output_file = fs::File::create(output_path)?;
        output_file.write_all(&decrypted_data)?;

        Ok(())
    }

    fn encrypt_aes(&self, data: &[u8], key: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        if key.len() != 32 {
            return Err(EncryptionError::InvalidKeyLength);
        }

        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
        let nonce = Nonce::generate(&mut OsRng);

        cipher
            .encrypt(&nonce, data)
            .map(|mut ciphertext| {
                let mut result = nonce.to_vec();
                result.append(&mut ciphertext);
                result
            })
            .map_err(|e| EncryptionError::CryptoError(e.to_string()))
    }

    fn decrypt_aes(&self, data: &[u8], key: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        if key.len() != 32 {
            return Err(EncryptionError::InvalidKeyLength);
        }

        if data.len() < 12 {
            return Err(EncryptionError::CryptoError("Invalid ciphertext".to_string()));
        }

        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
        let nonce = Nonce::from_slice(&data[..12]);
        let ciphertext = &data[12..];

        cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| EncryptionError::CryptoError(e.to_string()))
    }

    fn encrypt_chacha(&self, data: &[u8], key: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        if key.len() != 32 {
            return Err(EncryptionError::InvalidKeyLength);
        }

        let cipher = ChaCha20Poly1305::new(ChaChaKey::from_slice(key));
        let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);

        cipher
            .encrypt(&nonce, data)
            .map(|mut ciphertext| {
                let mut result = nonce.to_vec();
                result.append(&mut ciphertext);
                result
            })
            .map_err(|e| EncryptionError::CryptoError(e.to_string()))
    }

    fn decrypt_chacha(&self, data: &[u8], key: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        if key.len() != 32 {
            return Err(EncryptionError::InvalidKeyLength);
        }

        if data.len() < 12 {
            return Err(EncryptionError::CryptoError("Invalid ciphertext".to_string()));
        }

        let cipher = ChaCha20Poly1305::new(ChaChaKey::from_slice(key));
        let nonce = ChaChaNonce::from_slice(&data[..12]);
        let ciphertext = &data[12..];

        cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| EncryptionError::CryptoError(e.to_string()))
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
    fn test_aes_encryption_decryption() {
        let test_data = b"Hello, World! This is a test message.";
        let key = generate_random_key();

        let encryptor = FileEncryptor::new(EncryptionAlgorithm::Aes256Gcm);
        
        let encrypted = encryptor.encrypt_aes(test_data, &key).unwrap();
        let decrypted = encryptor.decrypt_aes(&encrypted, &key).unwrap();

        assert_eq!(test_data, decrypted.as_slice());
    }

    #[test]
    fn test_chacha_encryption_decryption() {
        let test_data = b"Another test message for ChaCha20Poly1305";
        let key = generate_random_key();

        let encryptor = FileEncryptor::new(EncryptionAlgorithm::ChaCha20Poly1305);
        
        let encrypted = encryptor.encrypt_chacha(test_data, &key).unwrap();
        let decrypted = encryptor.decrypt_chacha(&encrypted, &key).unwrap();

        assert_eq!(test_data, decrypted.as_slice());
    }

    #[test]
    fn test_file_encryption() {
        let original_content = b"File encryption test content";
        let key = generate_random_key();

        let input_file = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        fs::write(input_file.path(), original_content).unwrap();

        let encryptor = FileEncryptor::new(EncryptionAlgorithm::Aes256Gcm);
        
        encryptor
            .encrypt_file(input_file.path(), output_file.path(), &key)
            .unwrap();
        
        encryptor
            .decrypt_file(output_file.path(), decrypted_file.path(), &key)
            .unwrap();

        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(original_content, decrypted_content.as_slice());
    }
}
use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use pbkdf2::pbkdf2_hmac;
use rand::RngCore;
use sha2::Sha256;
use std::fs;
use std::io::{Read, Write};

type Aes256CbcEnc = cbc::Encryptor<aes::Aes256>;
type Aes256CbcDec = cbc::Decryptor<aes::Aes256>;

const SALT_LEN: usize = 16;
const IV_LEN: usize = 16;
const KEY_ITERATIONS: u32 = 100_000;
const KEY_LEN: usize = 32;

pub struct FileCrypto;

impl FileCrypto {
    pub fn encrypt_file(input_path: &str, output_path: &str, password: &str) -> Result<(), String> {
        let mut file_data = fs::read(input_path).map_err(|e| format!("Read failed: {}", e))?;
        
        let mut salt = [0u8; SALT_LEN];
        let mut iv = [0u8; IV_LEN];
        rand::thread_rng().fill_bytes(&mut salt);
        rand::thread_rng().fill_bytes(&mut iv);
        
        let key = Self::derive_key(password, &salt);
        
        let cipher = Aes256CbcEnc::new(&key.into(), &iv.into());
        let encrypted_data = cipher.encrypt_padded_vec_mut::<Pkcs7>(&file_data);
        
        let mut output = Vec::with_capacity(SALT_LEN + IV_LEN + encrypted_data.len());
        output.extend_from_slice(&salt);
        output.extend_from_slice(&iv);
        output.extend_from_slice(&encrypted_data);
        
        fs::write(output_path, &output).map_err(|e| format!("Write failed: {}", e))?;
        Ok(())
    }
    
    pub fn decrypt_file(input_path: &str, output_path: &str, password: &str) -> Result<(), String> {
        let encrypted_data = fs::read(input_path).map_err(|e| format!("Read failed: {}", e))?;
        
        if encrypted_data.len() < SALT_LEN + IV_LEN {
            return Err("Invalid encrypted file format".to_string());
        }
        
        let salt = &encrypted_data[0..SALT_LEN];
        let iv = &encrypted_data[SALT_LEN..SALT_LEN + IV_LEN];
        let ciphertext = &encrypted_data[SALT_LEN + IV_LEN..];
        
        let key = Self::derive_key(password, salt);
        
        let cipher = Aes256CbcDec::new(&key.into(), iv.into());
        let decrypted_data = cipher.decrypt_padded_vec_mut::<Pkcs7>(ciphertext)
            .map_err(|e| format!("Decryption failed: {}", e))?;
        
        fs::write(output_path, &decrypted_data).map_err(|e| format!("Write failed: {}", e))?;
        Ok(())
    }
    
    fn derive_key(password: &str, salt: &[u8]) -> [u8; KEY_LEN] {
        let mut key = [0u8; KEY_LEN];
        pbkdf2_hmac::<Sha256>(password.as_bytes(), salt, KEY_ITERATIONS, &mut key);
        key
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_encrypt_decrypt() {
        let original_data = b"Secret data that needs protection";
        let password = "strong_password_123";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), original_data).unwrap();
        
        FileCrypto::encrypt_file(
            input_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap(),
            password
        ).unwrap();
        
        FileCrypto::decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap(),
            password
        ).unwrap();
        
        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(original_data.as_slice(), decrypted_data.as_slice());
    }
    
    #[test]
    fn test_wrong_password() {
        let original_data = b"Test data";
        let password = "correct_password";
        let wrong_password = "wrong_password";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), original_data).unwrap();
        
        FileCrypto::encrypt_file(
            input_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap(),
            password
        ).unwrap();
        
        let result = FileCrypto::decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap(),
            wrong_password
        );
        
        assert!(result.is_err());
    }
}use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;

pub fn encrypt_file(input_path: &str, output_path: &str, key: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    let mut input_file = File::open(input_path)?;
    let mut buffer = Vec::new();
    input_file.read_to_end(&mut buffer)?;

    let encrypted_data: Vec<u8> = buffer
        .iter()
        .enumerate()
        .map(|(i, &byte)| byte ^ key[i % key.len()])
        .collect();

    let mut output_file = File::create(output_path)?;
    output_file.write_all(&encrypted_data)?;
    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str, key: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    encrypt_file(input_path, output_path, key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let key = b"secret_key";
        let original_content = b"Hello, this is a test message!";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        fs::write(input_file.path(), original_content).unwrap();

        encrypt_file(
            input_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap(),
            key,
        ).unwrap();

        decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap(),
            key,
        ).unwrap();

        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(original_content.to_vec(), decrypted_content);
    }
}
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

    pub fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let ciphertext = self
            .cipher
            .encrypt(&nonce, plaintext)
            .map_err(|e| format!("Encryption failed: {}", e))?;
        
        let mut result = Vec::with_capacity(nonce.len() + ciphertext.len());
        result.extend_from_slice(&nonce);
        result.extend_from_slice(&ciphertext);
        Ok(result)
    }

    pub fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
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

pub fn process_file_data(data: &[u8]) -> Result<(Vec<u8>, Vec<u8>), Box<dyn Error>> {
    let encryptor = FileEncryptor::new();
    let encrypted = encryptor.encrypt(data)?;
    let decrypted = encryptor.decrypt(&encrypted)?;
    
    if data != decrypted.as_slice() {
        return Err("Encryption/decryption mismatch".into());
    }
    
    Ok((encrypted, decrypted))
}
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: u8 = 0x55;

pub fn encrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    let encryption_key = key.unwrap_or(DEFAULT_KEY);
    
    let mut input_file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    input_file.read_to_end(&mut buffer)?;
    
    let encrypted_data: Vec<u8> = buffer.iter()
        .map(|byte| byte ^ encryption_key)
        .collect();
    
    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&encrypted_data)?;
    
    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    encrypt_file(input_path, output_path, key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_encryption_decryption() {
        let original_text = b"Hello, Rust!";
        let test_key = Some(0xAA);
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), original_text).unwrap();
        
        encrypt_file(
            input_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap(),
            test_key
        ).unwrap();
        
        let encrypted_content = fs::read(encrypted_file.path()).unwrap();
        assert_ne!(encrypted_content, original_text);
        
        decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap(),
            test_key
        ).unwrap();
        
        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(decrypted_content, original_text);
    }
}