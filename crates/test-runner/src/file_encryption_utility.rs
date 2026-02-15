
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHash, PasswordHasher, PasswordVerifier, SaltString
    },
    Argon2
};
use std::fs;
use std::io::{Read, Write};
use std::path::Path;

pub struct EncryptionResult {
    pub ciphertext: Vec<u8>,
    pub nonce: Vec<u8>,
    pub salt: String,
}

pub fn derive_key(password: &str, salt: &SaltString) -> Result<Key<Aes256Gcm>, String> {
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), salt)
        .map_err(|e| format!("Key derivation failed: {}", e))?;
    
    let hash_bytes = password_hash.hash.ok_or("No hash generated")?.as_bytes();
    if hash_bytes.len() < 32 {
        return Err("Derived key too short".to_string());
    }
    
    let key_slice = &hash_bytes[..32];
    Ok(*Key::<Aes256Gcm>::from_slice(key_slice))
}

pub fn encrypt_file(
    input_path: &Path,
    output_path: &Path,
    password: &str
) -> Result<EncryptionResult, String> {
    let mut file_data = Vec::new();
    fs::File::open(input_path)
        .map_err(|e| format!("Failed to open input file: {}", e))?
        .read_to_end(&mut file_data)
        .map_err(|e| format!("Failed to read file: {}", e))?;

    let salt = SaltString::generate(&mut OsRng);
    let key = derive_key(password, &salt)?;
    
    let cipher = Aes256Gcm::new(&key);
    let nonce_bytes: [u8; 12] = OsRng.fill_bytes();
    let nonce = Nonce::from_slice(&nonce_bytes);
    
    let ciphertext = cipher
        .encrypt(nonce, file_data.as_ref())
        .map_err(|e| format!("Encryption failed: {}", e))?;
    
    let mut output_file = fs::File::create(output_path)
        .map_err(|e| format!("Failed to create output file: {}", e))?;
    
    output_file.write_all(&ciphertext)
        .map_err(|e| format!("Failed to write encrypted data: {}", e))?;
    
    Ok(EncryptionResult {
        ciphertext,
        nonce: nonce_bytes.to_vec(),
        salt: salt.to_string(),
    })
}

pub fn decrypt_file(
    input_path: &Path,
    output_path: &Path,
    password: &str,
    salt_str: &str,
    nonce_bytes: &[u8]
) -> Result<Vec<u8>, String> {
    let mut encrypted_data = Vec::new();
    fs::File::open(input_path)
        .map_err(|e| format!("Failed to open encrypted file: {}", e))?
        .read_to_end(&mut encrypted_data)
        .map_err(|e| format!("Failed to read encrypted data: {}", e))?;

    let salt = SaltString::new(salt_str)
        .map_err(|e| format!("Invalid salt format: {}", e))?;
    
    let key = derive_key(password, &salt)?;
    let cipher = Aes256Gcm::new(&key);
    
    if nonce_bytes.len() != 12 {
        return Err("Invalid nonce length".to_string());
    }
    let nonce = Nonce::from_slice(nonce_bytes);
    
    let plaintext = cipher
        .decrypt(nonce, encrypted_data.as_ref())
        .map_err(|e| format!("Decryption failed: {}", e))?;
    
    fs::File::create(output_path)
        .map_err(|e| format!("Failed to create output file: {}", e))?
        .write_all(&plaintext)
        .map_err(|e| format!("Failed to write decrypted data: {}", e))?;
    
    Ok(plaintext)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_encryption_decryption() {
        let test_data = b"Secret data for encryption test";
        let password = "strong_password_123";
        
        let mut input_file = NamedTempFile::new().unwrap();
        input_file.write_all(test_data).unwrap();
        
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        let result = encrypt_file(
            input_file.path(),
            encrypted_file.path(),
            password
        ).unwrap();
        
        let decrypted = decrypt_file(
            encrypted_file.path(),
            decrypted_file.path(),
            password,
            &result.salt,
            &result.nonce
        ).unwrap();
        
        assert_eq!(decrypted, test_data);
    }
    
    #[test]
    fn test_wrong_password_fails() {
        let test_data = b"Test data";
        let password = "correct_password";
        let wrong_password = "wrong_password";
        
        let mut input_file = NamedTempFile::new().unwrap();
        input_file.write_all(test_data).unwrap();
        
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        let result = encrypt_file(
            input_file.path(),
            encrypted_file.path(),
            password
        ).unwrap();
        
        let decrypt_result = decrypt_file(
            encrypted_file.path(),
            decrypted_file.path(),
            wrong_password,
            &result.salt,
            &result.nonce
        );
        
        assert!(decrypt_result.is_err());
    }
}use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;
use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHasher, SaltString
    },
    Argon2
};

const NONCE_SIZE: usize = 12;
const SALT_SIZE: usize = 16;

pub struct FileEncryptor {
    cipher: Aes256Gcm,
}

impl FileEncryptor {
    pub fn new(password: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        
        let password_hash = argon2.hash_password(password.as_bytes(), &salt)?;
        let key_bytes = password_hash.hash.ok_or("Hash generation failed")?;
        
        let key = Key::<Aes256Gcm>::from_slice(key_bytes.as_bytes());
        let cipher = Aes256Gcm::new(key);
        
        Ok(Self { cipher })
    }

    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let mut input_file = File::open(input_path)?;
        let mut plaintext = Vec::new();
        input_file.read_to_end(&mut plaintext)?;

        let mut nonce_bytes = [0u8; NONCE_SIZE];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = self.cipher.encrypt(nonce, plaintext.as_ref())
            .map_err(|e| format!("Encryption failed: {}", e))?;

        let mut output_file = File::create(output_path)?;
        output_file.write_all(&nonce_bytes)?;
        output_file.write_all(&ciphertext)?;

        Ok(())
    }

    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let mut input_file = File::open(input_path)?;
        let mut encrypted_data = Vec::new();
        input_file.read_to_end(&mut encrypted_data)?;

        if encrypted_data.len() < NONCE_SIZE {
            return Err("Invalid encrypted file format".into());
        }

        let (nonce_bytes, ciphertext) = encrypted_data.split_at(NONCE_SIZE);
        let nonce = Nonce::from_slice(nonce_bytes);

        let plaintext = self.cipher.decrypt(nonce, ciphertext)
            .map_err(|e| format!("Decryption failed: {}", e))?;

        let mut output_file = File::create(output_path)?;
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
        let password = "secure_password_123";
        let encryptor = FileEncryptor::new(password).unwrap();

        let original_content = b"Secret data that needs protection";
        let mut input_file = NamedTempFile::new().unwrap();
        input_file.write_all(original_content).unwrap();

        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        encryptor.encrypt_file(input_file.path(), encrypted_file.path()).unwrap();
        encryptor.decrypt_file(encrypted_file.path(), decrypted_file.path()).unwrap();

        let mut decrypted_content = Vec::new();
        File::open(decrypted_file.path())
            .unwrap()
            .read_to_end(&mut decrypted_content)
            .unwrap();

        assert_eq!(decrypted_content, original_content);
    }
}use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use chacha20poly1305::{ChaCha20Poly1305, Key as ChaChaKey, Nonce as ChaChaNonce};
use std::error::Error;

pub enum CipherType {
    Aes256Gcm,
    ChaCha20Poly1305,
}

pub struct EncryptionResult {
    pub ciphertext: Vec<u8>,
    pub nonce: Vec<u8>,
}

pub fn encrypt_data(
    plaintext: &[u8],
    key: &[u8],
    cipher_type: CipherType,
) -> Result<EncryptionResult, Box<dyn Error>> {
    match cipher_type {
        CipherType::Aes256Gcm => {
            let key = Key::<Aes256Gcm>::from_slice(key);
            let cipher = Aes256Gcm::new(key);
            let nonce = Nonce::generate(&mut OsRng);
            let ciphertext = cipher
                .encrypt(&nonce, plaintext)
                .map_err(|e| format!("Encryption failed: {}", e))?;
            Ok(EncryptionResult {
                ciphertext,
                nonce: nonce.to_vec(),
            })
        }
        CipherType::ChaCha20Poly1305 => {
            let key = ChaChaKey::from_slice(key);
            let cipher = ChaCha20Poly1305::new(key);
            let nonce = ChaChaNonce::generate(&mut OsRng);
            let ciphertext = cipher
                .encrypt(&nonce, plaintext)
                .map_err(|e| format!("Encryption failed: {}", e))?;
            Ok(EncryptionResult {
                ciphertext,
                nonce: nonce.to_vec(),
            })
        }
    }
}

pub fn decrypt_data(
    ciphertext: &[u8],
    key: &[u8],
    nonce: &[u8],
    cipher_type: CipherType,
) -> Result<Vec<u8>, Box<dyn Error>> {
    match cipher_type {
        CipherType::Aes256Gcm => {
            let key = Key::<Aes256Gcm>::from_slice(key);
            let cipher = Aes256Gcm::new(key);
            let nonce = Nonce::from_slice(nonce);
            cipher
                .decrypt(nonce, ciphertext)
                .map_err(|e| format!("Decryption failed: {}", e).into())
        }
        CipherType::ChaCha20Poly1305 => {
            let key = ChaChaKey::from_slice(key);
            let cipher = ChaCha20Poly1305::new(key);
            let nonce = ChaChaNonce::from_slice(nonce);
            cipher
                .decrypt(nonce, ciphertext)
                .map_err(|e| format!("Decryption failed: {}", e).into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn generate_key() -> Vec<u8> {
        let mut key = vec![0u8; 32];
        OsRng.fill_bytes(&mut key);
        key
    }

    #[test]
    fn test_aes_encryption_decryption() {
        let key = generate_key();
        let plaintext = b"Secret message for AES";
        let result = encrypt_data(plaintext, &key, CipherType::Aes256Gcm).unwrap();
        let decrypted = decrypt_data(&result.ciphertext, &key, &result.nonce, CipherType::Aes256Gcm).unwrap();
        assert_eq!(plaintext.to_vec(), decrypted);
    }

    #[test]
    fn test_chacha_encryption_decryption() {
        let key = generate_key();
        let plaintext = b"Secret message for ChaCha";
        let result = encrypt_data(plaintext, &key, CipherType::ChaCha20Poly1305).unwrap();
        let decrypted = decrypt_data(&result.ciphertext, &key, &result.nonce, CipherType::ChaCha20Poly1305).unwrap();
        assert_eq!(plaintext.to_vec(), decrypted);
    }

    #[test]
    fn test_wrong_key_fails() {
        let key = generate_key();
        let wrong_key = generate_key();
        let plaintext = b"Test message";
        let result = encrypt_data(plaintext, &key, CipherType::Aes256Gcm).unwrap();
        let decryption_result = decrypt_data(&result.ciphertext, &wrong_key, &result.nonce, CipherType::Aes256Gcm);
        assert!(decryption_result.is_err());
    }
}
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use argon2::{
    password_hash::{rand_core::OsRng as ArgonRng, PasswordHasher, SaltString},
    Argon2
};
use std::fs;
use std::io::{Read, Write};
use std::path::Path;

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

    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let mut file_data = Vec::new();
        fs::File::open(input_path)?.read_to_end(&mut file_data)?;

        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&self.key));
        let nonce = Nonce::from_slice(&rand::random::<[u8; NONCE_SIZE]>());

        let ciphertext = cipher.encrypt(nonce, file_data.as_ref())
            .map_err(|e| format!("Encryption failed: {}", e))?;

        let mut output = fs::File::create(output_path)?;
        output.write_all(nonce)?;
        output.write_all(&ciphertext)?;

        Ok(())
    }

    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let mut encrypted_data = Vec::new();
        fs::File::open(input_path)?.read_to_end(&mut encrypted_data)?;

        if encrypted_data.len() < NONCE_SIZE {
            return Err("File too short to contain nonce".into());
        }

        let nonce = Nonce::from_slice(&encrypted_data[..NONCE_SIZE]);
        let ciphertext = &encrypted_data[NONCE_SIZE..];

        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&self.key));
        let plaintext = cipher.decrypt(nonce, ciphertext)
            .map_err(|e| format!("Decryption failed: {}", e))?;

        fs::File::create(output_path)?.write_all(&plaintext)?;

        Ok(())
    }
}

pub fn generate_random_key() -> [u8; 32] {
    rand::random::<[u8; 32]>()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let password = "test_password_123!";
        let encryptor = FileEncryptor::new(password).unwrap();
        
        let test_data = b"Hello, this is a secret message!";
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        fs::write(input_file.path(), test_data).unwrap();
        
        encryptor.encrypt_file(input_file.path(), encrypted_file.path()).unwrap();
        encryptor.decrypt_file(encrypted_file.path(), decrypted_file.path()).unwrap();
        
        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(test_data.to_vec(), decrypted_data);
    }
}