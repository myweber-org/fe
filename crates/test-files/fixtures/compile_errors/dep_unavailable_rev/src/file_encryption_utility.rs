
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use argon2::{Argon2, PasswordHasher, PasswordVerifier};
use rand::RngCore;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;

const NONCE_LENGTH: usize = 12;
const SALT_LENGTH: usize = 16;
const TAG_LENGTH: usize = 16;

pub struct EncryptionResult {
    pub ciphertext: Vec<u8>,
    pub nonce: [u8; NONCE_LENGTH],
    pub salt: [u8; SALT_LENGTH],
}

pub fn derive_key(password: &str, salt: &[u8]) -> Result<Key<Aes256Gcm>, String> {
    let argon2 = Argon2::default();
    let mut key = [0u8; 32];
    
    argon2
        .hash_password_into(password.as_bytes(), salt, &mut key)
        .map_err(|e| format!("Key derivation failed: {}", e))?;
    
    Ok(Key::<Aes256Gcm>::from_slice(&key).clone())
}

pub fn encrypt_file(
    input_path: &Path,
    output_path: &Path,
    password: &str,
) -> Result<EncryptionResult, String> {
    let mut input_file = File::open(input_path)
        .map_err(|e| format!("Failed to open input file: {}", e))?;
    
    let mut plaintext = Vec::new();
    input_file
        .read_to_end(&mut plaintext)
        .map_err(|e| format!("Failed to read input file: {}", e))?;
    
    let mut rng = rand::thread_rng();
    let mut salt = [0u8; SALT_LENGTH];
    let mut nonce_bytes = [0u8; NONCE_LENGTH];
    
    rng.fill_bytes(&mut salt);
    rng.fill_bytes(&mut nonce_bytes);
    
    let key = derive_key(password, &salt)?;
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(&nonce_bytes);
    
    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_ref())
        .map_err(|e| format!("Encryption failed: {}", e))?;
    
    let mut output_file = File::create(output_path)
        .map_err(|e| format!("Failed to create output file: {}", e))?;
    
    output_file
        .write_all(&salt)
        .map_err(|e| format!("Failed to write salt: {}", e))?;
    output_file
        .write_all(&nonce_bytes)
        .map_err(|e| format!("Failed to write nonce: {}", e))?;
    output_file
        .write_all(&ciphertext)
        .map_err(|e| format!("Failed to write ciphertext: {}", e))?;
    
    Ok(EncryptionResult {
        ciphertext,
        nonce: nonce_bytes,
        salt,
    })
}

pub fn decrypt_file(
    input_path: &Path,
    output_path: &Path,
    password: &str,
) -> Result<Vec<u8>, String> {
    let mut input_file = File::open(input_path)
        .map_err(|e| format!("Failed to open input file: {}", e))?;
    
    let mut encrypted_data = Vec::new();
    input_file
        .read_to_end(&mut encrypted_data)
        .map_err(|e| format!("Failed to read input file: {}", e))?;
    
    if encrypted_data.len() < SALT_LENGTH + NONCE_LENGTH + TAG_LENGTH {
        return Err("File too short to contain valid encrypted data".to_string());
    }
    
    let salt = &encrypted_data[0..SALT_LENGTH];
    let nonce_bytes = &encrypted_data[SALT_LENGTH..SALT_LENGTH + NONCE_LENGTH];
    let ciphertext = &encrypted_data[SALT_LENGTH + NONCE_LENGTH..];
    
    let key = derive_key(password, salt)?;
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(nonce_bytes);
    
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| format!("Decryption failed: {}", e))?;
    
    let mut output_file = File::create(output_path)
        .map_err(|e| format!("Failed to create output file: {}", e))?;
    
    output_file
        .write_all(&plaintext)
        .map_err(|e| format!("Failed to write decrypted data: {}", e))?;
    
    Ok(plaintext)
}

pub fn generate_random_file(path: &Path, size: usize) -> Result<(), String> {
    let mut file = File::create(path)
        .map_err(|e| format!("Failed to create test file: {}", e))?;
    
    let mut rng = rand::thread_rng();
    let mut buffer = vec![0u8; size];
    rng.fill_bytes(&mut buffer);
    
    file.write_all(&buffer)
        .map_err(|e| format!("Failed to write test data: {}", e))?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_encryption_decryption() {
        let plaintext_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        let test_data = b"Test encryption and decryption data";
        fs::write(plaintext_file.path(), test_data).unwrap();
        
        let password = "secure_password_123";
        
        let encrypt_result = encrypt_file(
            plaintext_file.path(),
            encrypted_file.path(),
            password,
        );
        assert!(encrypt_result.is_ok());
        
        let decrypt_result = decrypt_file(
            encrypted_file.path(),
            decrypted_file.path(),
            password,
        );
        assert!(decrypt_result.is_ok());
        
        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(decrypted_data, test_data);
    }
    
    #[test]
    fn test_wrong_password() {
        let plaintext_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        let test_data = b"Sensitive information";
        fs::write(plaintext_file.path(), test_data).unwrap();
        
        let encrypt_result = encrypt_file(
            plaintext_file.path(),
            encrypted_file.path(),
            "correct_password",
        );
        assert!(encrypt_result.is_ok());
        
        let decrypt_result = decrypt_file(
            encrypted_file.path(),
            decrypted_file.path(),
            "wrong_password",
        );
        assert!(decrypt_result.is_err());
    }
}use aes_gcm::{
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
    EncryptionError(String),
    DecryptionError(String),
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
            EncryptionAlgorithm::Aes256Gcm => self.aes_encrypt(&file_data, key)?,
            EncryptionAlgorithm::ChaCha20Poly1305 => self.chacha_encrypt(&file_data, key)?,
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
            EncryptionAlgorithm::Aes256Gcm => self.aes_decrypt(&encrypted_data, key)?,
            EncryptionAlgorithm::ChaCha20Poly1305 => self.chacha_decrypt(&encrypted_data, key)?,
        };

        let mut output_file = fs::File::create(output_path)?;
        output_file.write_all(&decrypted_data)?;

        Ok(())
    }

    fn aes_encrypt(&self, data: &[u8], key: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        if key.len() != 32 {
            return Err(EncryptionError::EncryptionError(
                "AES-256-GCM requires 32-byte key".to_string(),
            ));
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
            .map_err(|e| EncryptionError::EncryptionError(e.to_string()))
    }

    fn aes_decrypt(&self, data: &[u8], key: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        if key.len() != 32 {
            return Err(EncryptionError::DecryptionError(
                "AES-256-GCM requires 32-byte key".to_string(),
            ));
        }

        if data.len() < 12 {
            return Err(EncryptionError::DecryptionError(
                "Encrypted data too short".to_string(),
            ));
        }

        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
        let nonce = Nonce::from_slice(&data[..12]);
        let ciphertext = &data[12..];

        cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| EncryptionError::DecryptionError(e.to_string()))
    }

    fn chacha_encrypt(&self, data: &[u8], key: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        if key.len() != 32 {
            return Err(EncryptionError::EncryptionError(
                "ChaCha20Poly1305 requires 32-byte key".to_string(),
            ));
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
            .map_err(|e| EncryptionError::EncryptionError(e.to_string()))
    }

    fn chacha_decrypt(&self, data: &[u8], key: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        if key.len() != 32 {
            return Err(EncryptionError::DecryptionError(
                "ChaCha20Poly1305 requires 32-byte key".to_string(),
            ));
        }

        if data.len() < 12 {
            return Err(EncryptionError::DecryptionError(
                "Encrypted data too short".to_string(),
            ));
        }

        let cipher = ChaCha20Poly1305::new(ChaChaKey::from_slice(key));
        let nonce = ChaChaNonce::from_slice(&data[..12]);
        let ciphertext = &data[12..];

        cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| EncryptionError::DecryptionError(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_aes_encryption_decryption() {
        let test_data = b"Hello, World! This is a test message.";
        let key = [0u8; 32];

        let encryptor = FileEncryptor::new(EncryptionAlgorithm::Aes256Gcm);
        
        let encrypted = encryptor.aes_encrypt(test_data, &key).unwrap();
        let decrypted = encryptor.aes_decrypt(&encrypted, &key).unwrap();
        
        assert_eq!(test_data.to_vec(), decrypted);
    }

    #[test]
    fn test_chacha_encryption_decryption() {
        let test_data = b"Another test message for ChaCha20Poly1305";
        let key = [1u8; 32];

        let encryptor = FileEncryptor::new(EncryptionAlgorithm::ChaCha20Poly1305);
        
        let encrypted = encryptor.chacha_encrypt(test_data, &key).unwrap();
        let decrypted = encryptor.chacha_decrypt(&encrypted, &key).unwrap();
        
        assert_eq!(test_data.to_vec(), decrypted);
    }

    #[test]
    fn test_file_encryption() {
        let input_file = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        let test_data = b"File encryption test data";
        fs::write(input_file.path(), test_data).unwrap();
        
        let key = [2u8; 32];
        let encryptor = FileEncryptor::new(EncryptionAlgorithm::Aes256Gcm);
        
        encryptor.encrypt_file(input_file.path(), output_file.path(), &key).unwrap();
        encryptor.decrypt_file(output_file.path(), decrypted_file.path(), &key).unwrap();
        
        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(test_data.to_vec(), decrypted_data);
    }
}