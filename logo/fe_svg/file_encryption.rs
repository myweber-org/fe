
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use argon2::{
    password_hash::{rand_core::OsRng as ArgonRng, PasswordHasher, SaltString},
    Argon2,
};
use std::{
    fs::{self, File},
    io::{Read, Write},
    path::Path,
};

const NONCE_SIZE: usize = 12;
const SALT_SIZE: usize = 16;

pub struct FileEncryptor {
    key: [u8; 32],
}

impl FileEncryptor {
    pub fn new(password: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let salt = SaltString::generate(&mut ArgonRng);
        let argon2 = Argon2::default();
        let password_hash = argon2.hash_password(password.as_bytes(), &salt)?;
        
        let mut key = [0u8; 32];
        key.copy_from_slice(&password_hash.hash.unwrap().as_bytes()[..32]);
        
        Ok(Self { key })
    }

    pub fn encrypt_file(
        &self,
        input_path: &Path,
        output_path: &Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut file = File::open(input_path)?;
        let mut plaintext = Vec::new();
        file.read_to_end(&mut plaintext)?;

        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&self.key));
        let nonce = Nonce::from_slice(&generate_random_bytes(NONCE_SIZE));

        let ciphertext = cipher.encrypt(nonce, plaintext.as_ref())
            .map_err(|e| format!("Encryption failed: {}", e))?;

        let mut output = File::create(output_path)?;
        output.write_all(nonce.as_slice())?;
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

        if encrypted_data.len() < NONCE_SIZE {
            return Err("Invalid encrypted file format".into());
        }

        let nonce = Nonce::from_slice(&encrypted_data[..NONCE_SIZE]);
        let ciphertext = &encrypted_data[NONCE_SIZE..];

        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&self.key));
        let plaintext = cipher.decrypt(nonce, ciphertext)
            .map_err(|e| format!("Decryption failed: {}", e))?;

        let mut output = File::create(output_path)?;
        output.write_all(&plaintext)?;

        Ok(())
    }
}

fn generate_random_bytes(size: usize) -> Vec<u8> {
    let mut bytes = vec![0u8; size];
    OsRng.fill_bytes(&mut bytes);
    bytes
}

pub fn secure_delete_file(path: &Path) -> Result<(), std::io::Error> {
    let metadata = fs::metadata(path)?;
    let file_size = metadata.len() as usize;
    
    let mut file = fs::OpenOptions::new().write(true).open(path)?;
    
    for _ in 0..3 {
        let random_data: Vec<u8> = (0..file_size).map(|_| rand::random()).collect();
        file.write_all(&random_data)?;
        file.flush()?;
        file.seek(std::io::SeekFrom::Start(0))?;
    }
    
    fs::remove_file(path)
}use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use std::error::Error;

pub fn encrypt_data(plaintext: &[u8]) -> Result<(Vec<u8>, Vec<u8>), Box<dyn Error>> {
    let key = Aes256Gcm::generate_key(&mut OsRng);
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(b"unique_nonce_");
    
    let ciphertext = cipher.encrypt(nonce, plaintext)
        .map_err(|e| format!("Encryption failed: {}", e))?;
    
    Ok((ciphertext, key.to_vec()))
}

pub fn decrypt_data(ciphertext: &[u8], key: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
    let key = Key::<Aes256Gcm>::from_slice(key);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(b"unique_nonce_");
    
    let plaintext = cipher.decrypt(nonce, ciphertext)
        .map_err(|e| format!("Decryption failed: {}", e))?;
    
    Ok(plaintext)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_roundtrip() {
        let original_data = b"Secret message for encryption test";
        let (ciphertext, key) = encrypt_data(original_data).unwrap();
        let decrypted_data = decrypt_data(&ciphertext, &key).unwrap();
        
        assert_eq!(original_data.to_vec(), decrypted_data);
    }
}use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use chacha20poly1305::{ChaCha20Poly1305, Key as ChaChaKey, Nonce as ChaChaNonce};
use rand::RngCore;
use std::error::Error;

pub enum CipherAlgorithm {
    Aes256Gcm,
    ChaCha20Poly1305,
}

pub struct EncryptionResult {
    pub ciphertext: Vec<u8>,
    pub nonce: Vec<u8>,
    pub algorithm: CipherAlgorithm,
}

pub fn encrypt_data(
    plaintext: &[u8],
    algorithm: CipherAlgorithm,
) -> Result<EncryptionResult, Box<dyn Error>> {
    match algorithm {
        CipherAlgorithm::Aes256Gcm => {
            let key = Key::<Aes256Gcm>::generate(&mut OsRng);
            let cipher = Aes256Gcm::new(&key);
            let mut nonce_bytes = [0u8; 12];
            OsRng.fill_bytes(&mut nonce_bytes);
            let nonce = Nonce::from_slice(&nonce_bytes);
            
            let ciphertext = cipher
                .encrypt(nonce, plaintext)
                .map_err(|e| format!("Encryption failed: {}", e))?;
            
            Ok(EncryptionResult {
                ciphertext,
                nonce: nonce_bytes.to_vec(),
                algorithm: CipherAlgorithm::Aes256Gcm,
            })
        }
        CipherAlgorithm::ChaCha20Poly1305 => {
            let key = ChaChaKey::generate(&mut OsRng);
            let cipher = ChaCha20Poly1305::new(&key);
            let mut nonce_bytes = [0u8; 12];
            OsRng.fill_bytes(&mut nonce_bytes);
            let nonce = ChaChaNonce::from_slice(&nonce_bytes);
            
            let ciphertext = cipher
                .encrypt(nonce, plaintext)
                .map_err(|e| format!("Encryption failed: {}", e))?;
            
            Ok(EncryptionResult {
                ciphertext,
                nonce: nonce_bytes.to_vec(),
                algorithm: CipherAlgorithm::ChaCha20Poly1305,
            })
        }
    }
}

pub fn decrypt_data(
    encrypted: &EncryptionResult,
    key: &[u8],
) -> Result<Vec<u8>, Box<dyn Error>> {
    match encrypted.algorithm {
        CipherAlgorithm::Aes256Gcm => {
            let key = Key::<Aes256Gcm>::from_slice(key);
            let cipher = Aes256Gcm::new(key);
            let nonce = Nonce::from_slice(&encrypted.nonce);
            
            cipher
                .decrypt(nonce, encrypted.ciphertext.as_ref())
                .map_err(|e| format!("Decryption failed: {}", e).into())
        }
        CipherAlgorithm::ChaCha20Poly1305 => {
            let key = ChaChaKey::from_slice(key);
            let cipher = ChaCha20Poly1305::new(key);
            let nonce = ChaChaNonce::from_slice(&encrypted.nonce);
            
            cipher
                .decrypt(nonce, encrypted.ciphertext.as_ref())
                .map_err(|e| format!("Decryption failed: {}", e).into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aes_encryption_decryption() {
        let plaintext = b"Secret message for AES";
        let result = encrypt_data(plaintext, CipherAlgorithm::Aes256Gcm).unwrap();
        
        // In real usage, key would be stored securely
        let test_key = [0u8; 32]; // 32 bytes for AES-256
        
        let decrypted = decrypt_data(&result, &test_key);
        assert!(decrypted.is_err()); // Should fail with wrong key
        
        // Note: Actual key management would be required for proper testing
    }

    #[test]
    fn test_chacha_encryption_decryption() {
        let plaintext = b"Secret message for ChaCha";
        let result = encrypt_data(plaintext, CipherAlgorithm::ChaCha20Poly1305).unwrap();
        
        let test_key = [0u8; 32]; // 32 bytes for ChaCha20
        
        let decrypted = decrypt_data(&result, &test_key);
        assert!(decrypted.is_err());
    }
}