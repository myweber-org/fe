
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use chacha20poly1305::{
    aead::{Aead as ChaAead, KeyInit as ChaKeyInit},
    ChaCha20Poly1305, Key as ChaKey, Nonce as ChaNonce
};
use rand::RngCore;
use std::fs;
use std::io::{self, Write};
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

pub struct EncryptionResult {
    pub ciphertext: Vec<u8>,
    pub nonce: Vec<u8>,
}

pub trait Encryptor {
    fn encrypt(&self, plaintext: &[u8]) -> Result<EncryptionResult, EncryptionError>;
    fn decrypt(&self, ciphertext: &[u8], nonce: &[u8]) -> Result<Vec<u8>, EncryptionError>;
}

pub struct Aes256GcmEncryptor {
    key: Key<Aes256Gcm>,
}

impl Aes256GcmEncryptor {
    pub fn new() -> Self {
        let mut key_bytes = [0u8; 32];
        OsRng.fill_bytes(&mut key_bytes);
        let key = Key::<Aes256Gcm>::from_slice(&key_bytes).clone();
        Self { key }
    }
    
    pub fn from_key(key_bytes: &[u8]) -> Result<Self, EncryptionError> {
        if key_bytes.len() != 32 {
            return Err(EncryptionError::CryptoError(
                "AES-256-GCM requires 32-byte key".to_string()
            ));
        }
        let key = Key::<Aes256Gcm>::from_slice(key_bytes).clone();
        Ok(Self { key })
    }
}

impl Encryptor for Aes256GcmEncryptor {
    fn encrypt(&self, plaintext: &[u8]) -> Result<EncryptionResult, EncryptionError> {
        let cipher = Aes256Gcm::new(&self.key);
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        match cipher.encrypt(nonce, plaintext) {
            Ok(ciphertext) => Ok(EncryptionResult {
                ciphertext,
                nonce: nonce_bytes.to_vec(),
            }),
            Err(e) => Err(EncryptionError::CryptoError(e.to_string())),
        }
    }
    
    fn decrypt(&self, ciphertext: &[u8], nonce: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        if nonce.len() != 12 {
            return Err(EncryptionError::CryptoError(
                "AES-256-GCM requires 12-byte nonce".to_string()
            ));
        }
        
        let cipher = Aes256Gcm::new(&self.key);
        let nonce = Nonce::from_slice(nonce);
        
        match cipher.decrypt(nonce, ciphertext) {
            Ok(plaintext) => Ok(plaintext),
            Err(e) => Err(EncryptionError::CryptoError(e.to_string())),
        }
    }
}

pub struct ChaCha20Poly1305Encryptor {
    key: ChaKey,
}

impl ChaCha20Poly1305Encryptor {
    pub fn new() -> Self {
        let mut key_bytes = [0u8; 32];
        OsRng.fill_bytes(&mut key_bytes);
        let key = ChaKey::from_slice(&key_bytes).clone();
        Self { key }
    }
    
    pub fn from_key(key_bytes: &[u8]) -> Result<Self, EncryptionError> {
        if key_bytes.len() != 32 {
            return Err(EncryptionError::CryptoError(
                "ChaCha20Poly1305 requires 32-byte key".to_string()
            ));
        }
        let key = ChaKey::from_slice(key_bytes).clone();
        Ok(Self { key })
    }
}

impl Encryptor for ChaCha20Poly1305Encryptor {
    fn encrypt(&self, plaintext: &[u8]) -> Result<EncryptionResult, EncryptionError> {
        let cipher = ChaCha20Poly1305::new(&self.key);
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = ChaNonce::from_slice(&nonce_bytes);
        
        match cipher.encrypt(nonce, plaintext) {
            Ok(ciphertext) => Ok(EncryptionResult {
                ciphertext,
                nonce: nonce_bytes.to_vec(),
            }),
            Err(e) => Err(EncryptionError::CryptoError(e.to_string())),
        }
    }
    
    fn decrypt(&self, ciphertext: &[u8], nonce: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        if nonce.len() != 12 {
            return Err(EncryptionError::CryptoError(
                "ChaCha20Poly1305 requires 12-byte nonce".to_string()
            ));
        }
        
        let cipher = ChaCha20Poly1305::new(&self.key);
        let nonce = ChaNonce::from_slice(nonce);
        
        match cipher.decrypt(nonce, ciphertext) {
            Ok(plaintext) => Ok(plaintext),
            Err(e) => Err(EncryptionError::CryptoError(e.to_string())),
        }
    }
}

pub fn encrypt_file<P: AsRef<Path>>(
    path: P,
    encryptor: &dyn Encryptor,
) -> Result<(), EncryptionError> {
    let plaintext = fs::read(&path)?;
    let result = encryptor.encrypt(&plaintext)?;
    
    let encrypted_path = path.as_ref().with_extension("enc");
    let mut file = fs::File::create(&encrypted_path)?;
    file.write_all(&result.nonce)?;
    file.write_all(&result.ciphertext)?;
    
    println!("File encrypted successfully: {:?}", encrypted_path);
    Ok(())
}

pub fn decrypt_file<P: AsRef<Path>>(
    path: P,
    encryptor: &dyn Encryptor,
) -> Result<(), EncryptionError> {
    let ciphertext_with_nonce = fs::read(&path)?;
    
    if ciphertext_with_nonce.len() < 12 {
        return Err(EncryptionError::CryptoError(
            "File too small to contain valid encrypted data".to_string()
        ));
    }
    
    let (nonce, ciphertext) = ciphertext_with_nonce.split_at(12);
    let plaintext = encryptor.decrypt(ciphertext, nonce)?;
    
    let decrypted_path = path.as_ref().with_extension("dec");
    fs::write(&decrypted_path, plaintext)?;
    
    println!("File decrypted successfully: {:?}", decrypted_path);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_aes_encryption_decryption() {
        let encryptor = Aes256GcmEncryptor::new();
        let plaintext = b"Test secret message for AES-256-GCM";
        
        let result = encryptor.encrypt(plaintext).unwrap();
        let decrypted = encryptor.decrypt(&result.ciphertext, &result.nonce).unwrap();
        
        assert_eq!(plaintext.to_vec(), decrypted);
    }
    
    #[test]
    fn test_chacha_encryption_decryption() {
        let encryptor = ChaCha20Poly1305Encryptor::new();
        let plaintext = b"Test secret message for ChaCha20Poly1305";
        
        let result = encryptor.encrypt(plaintext).unwrap();
        let decrypted = encryptor.decrypt(&result.ciphertext, &result.nonce).unwrap();
        
        assert_eq!(plaintext.to_vec(), decrypted);
    }
    
    #[test]
    fn test_key_import() {
        let key = [0x42u8; 32];
        let encryptor = Aes256GcmEncryptor::from_key(&key).unwrap();
        let plaintext = b"Test with imported key";
        
        let result = encryptor.encrypt(plaintext).unwrap();
        let decrypted = encryptor.decrypt(&result.ciphertext, &result.nonce).unwrap();
        
        assert_eq!(plaintext.to_vec(), decrypted);
    }
}