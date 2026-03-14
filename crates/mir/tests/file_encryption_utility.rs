use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use argon2::{
    password_hash::{rand_core::OsRng as ArgonRng, PasswordHasher, SaltString},
    Argon2,
};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;

const NONCE_SIZE: usize = 12;
const SALT_SIZE: usize = 16;

pub struct EncryptionResult {
    pub ciphertext: Vec<u8>,
    pub nonce: [u8; NONCE_SIZE],
    pub salt: [u8; SALT_SIZE],
}

pub fn derive_key(password: &str, salt: &[u8]) -> Result<[u8; 32], Box<dyn std::error::Error>> {
    let salt_str = SaltString::encode_b64(salt)?;
    let argon2 = Argon2::default();
    let password_hash = argon2.hash_password(password.as_bytes(), &salt_str)?;
    
    let mut key = [0u8; 32];
    key.copy_from_slice(&password_hash.hash.unwrap().as_bytes()[..32]);
    Ok(key)
}

pub fn encrypt_file(
    input_path: &Path,
    output_path: &Path,
    password: &str,
) -> Result<EncryptionResult, Box<dyn std::error::Error>> {
    let mut file = File::open(input_path)?;
    let mut plaintext = Vec::new();
    file.read_to_end(&mut plaintext)?;
    
    let mut rng = OsRng;
    let mut nonce = [0u8; NONCE_SIZE];
    rng.fill_bytes(&mut nonce);
    
    let mut salt = [0u8; SALT_SIZE];
    ArgonRng.fill_bytes(&mut salt);
    
    let key = derive_key(password, &salt)?;
    let cipher = Aes256Gcm::new_from_slice(&key)?;
    
    let ciphertext = cipher
        .encrypt(Nonce::from_slice(&nonce), plaintext.as_ref())
        .map_err(|e| format!("Encryption failed: {}", e))?;
    
    let mut output_file = File::create(output_path)?;
    output_file.write_all(&ciphertext)?;
    
    Ok(EncryptionResult {
        ciphertext,
        nonce,
        salt,
    })
}

pub fn decrypt_file(
    input_path: &Path,
    output_path: &Path,
    password: &str,
    nonce: &[u8; NONCE_SIZE],
    salt: &[u8; SALT_SIZE],
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut file = File::open(input_path)?;
    let mut ciphertext = Vec::new();
    file.read_to_end(&mut ciphertext)?;
    
    let key = derive_key(password, salt)?;
    let cipher = Aes256Gcm::new_from_slice(&key)?;
    
    let plaintext = cipher
        .decrypt(Nonce::from_slice(nonce), ciphertext.as_ref())
        .map_err(|e| format!("Decryption failed: {}", e))?;
    
    let mut output_file = File::create(output_path)?;
    output_file.write_all(&plaintext)?;
    
    Ok(plaintext)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_encryption_decryption() {
        let test_data = b"Secret data that needs protection";
        let password = "strong_password_123";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), test_data).unwrap();
        
        let enc_result = encrypt_file(input_file.path(), encrypted_file.path(), password)
            .expect("Encryption should succeed");
        
        let decrypted = decrypt_file(
            encrypted_file.path(),
            decrypted_file.path(),
            password,
            &enc_result.nonce,
            &enc_result.salt,
        ).expect("Decryption should succeed");
        
        assert_eq!(decrypted, test_data);
    }
    
    #[test]
    fn test_wrong_password_fails() {
        let test_data = b"Test data";
        let password = "correct_password";
        let wrong_password = "wrong_password";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), test_data).unwrap();
        
        let enc_result = encrypt_file(input_file.path(), encrypted_file.path(), password)
            .expect("Encryption should succeed");
        
        let result = decrypt_file(
            encrypted_file.path(),
            decrypted_file.path(),
            wrong_password,
            &enc_result.nonce,
            &enc_result.salt,
        );
        
        assert!(result.is_err());
    }
}use aes_gcm::{
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
use std::io::{self, Write};

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
        
        Ok(Self { cipher })
    }
    
    pub fn encrypt_file(&self, input_path: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let plaintext = fs::read(input_path)?;
        let nonce = Nonce::from_slice(&generate_nonce());
        
        let ciphertext = self.cipher.encrypt(nonce, plaintext.as_ref())
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        
        let mut output = Vec::new();
        output.extend_from_slice(nonce);
        output.extend_from_slice(&ciphertext);
        
        fs::write(output_path, output)?;
        Ok(())
    }
    
    pub fn decrypt_file(&self, input_path: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let data = fs::read(input_path)?;
        if data.len() < NONCE_SIZE {
            return Err("File too short".into());
        }
        
        let (nonce_bytes, ciphertext) = data.split_at(NONCE_SIZE);
        let nonce = Nonce::from_slice(nonce_bytes);
        
        let plaintext = self.cipher.decrypt(nonce, ciphertext)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        
        fs::write(output_path, plaintext)?;
        Ok(())
    }
}

fn generate_nonce() -> [u8; NONCE_SIZE] {
    let mut nonce = [0u8; NONCE_SIZE];
    OsRng.fill_bytes(&mut nonce);
    nonce
}

pub fn encrypt_with_password(password: &str, input_file: &str, output_file: &str) -> Result<(), Box<dyn std::error::Error>> {
    let encryptor = FileEncryptor::from_password(password)?;
    encryptor.encrypt_file(input_file, output_file)
}

pub fn decrypt_with_password(password: &str, input_file: &str, output_file: &str) -> Result<(), Box<dyn std::error::Error>> {
    let encryptor = FileEncryptor::from_password(password)?;
    encryptor.decrypt_file(input_file, output_file)
}use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

pub struct XorCipher {
    key: Vec<u8>,
    key_position: usize,
}

impl XorCipher {
    pub fn new(key: &str) -> Self {
        XorCipher {
            key: key.as_bytes().to_vec(),
            key_position: 0,
        }
    }

    pub fn encrypt_bytes(&mut self, data: &[u8]) -> Vec<u8> {
        self.process_bytes(data)
    }

    pub fn decrypt_bytes(&mut self, data: &[u8]) -> Vec<u8> {
        self.process_bytes(data)
    }

    fn process_bytes(&mut self, data: &[u8]) -> Vec<u8> {
        let mut result = Vec::with_capacity(data.len());
        
        for &byte in data {
            let key_byte = self.key[self.key_position];
            result.push(byte ^ key_byte);
            
            self.key_position = (self.key_position + 1) % self.key.len();
        }
        
        result
    }

    pub fn reset(&mut self) {
        self.key_position = 0;
    }
}

pub fn encrypt_file(input_path: &Path, output_path: &Path, key: &str) -> io::Result<()> {
    let mut cipher = XorCipher::new(key);
    
    let mut input_file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    input_file.read_to_end(&mut buffer)?;
    
    let encrypted_data = cipher.encrypt_bytes(&buffer);
    
    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&encrypted_data)?;
    
    Ok(())
}

pub fn decrypt_file(input_path: &Path, output_path: &Path, key: &str) -> io::Result<()> {
    let mut cipher = XorCipher::new(key);
    
    let mut input_file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    input_file.read_to_end(&mut buffer)?;
    
    let decrypted_data = cipher.decrypt_bytes(&buffer);
    
    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&decrypted_data)?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_xor_cipher_symmetry() {
        let key = "secret_key";
        let original_data = b"Hello, World! This is a test message.";
        
        let mut cipher = XorCipher::new(key);
        let encrypted = cipher.encrypt_bytes(original_data);
        
        cipher.reset();
        let decrypted = cipher.decrypt_bytes(&encrypted);
        
        assert_eq!(original_data.to_vec(), decrypted);
    }

    #[test]
    fn test_file_encryption() -> io::Result<()> {
        let key = "test_key_123";
        let test_content = b"Sample file content for encryption test.";
        
        let mut input_file = NamedTempFile::new()?;
        input_file.write_all(test_content)?;
        let input_path = input_file.path();
        
        let mut encrypted_file = NamedTempFile::new()?;
        let encrypted_path = encrypted_file.path();
        
        encrypt_file(input_path, encrypted_path, key)?;
        
        let mut decrypted_file = NamedTempFile::new()?;
        let decrypted_path = decrypted_file.path();
        
        decrypt_file(encrypted_path, decrypted_path, key)?;
        
        let decrypted_content = fs::read(decrypted_path)?;
        assert_eq!(test_content.to_vec(), decrypted_content);
        
        Ok(())
    }
}use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce
};
use std::fs;
use std::io::{self, Write};
use std::path::Path;

const NONCE_SIZE: usize = 12;

pub fn encrypt_file(input_path: &Path, output_path: &Path) -> io::Result<()> {
    let key = Aes256Gcm::generate_key(&mut OsRng);
    let cipher = Aes256Gcm::new(&key);
    
    let plaintext = fs::read(input_path)?;
    let nonce = Nonce::from_slice(&[0u8; NONCE_SIZE]);
    
    let ciphertext = cipher.encrypt(nonce, plaintext.as_ref())
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    
    let mut output = fs::File::create(output_path)?;
    output.write_all(&key)?;
    output.write_all(nonce)?;
    output.write_all(&ciphertext)?;
    
    Ok(())
}

pub fn decrypt_file(input_path: &Path, output_path: &Path) -> io::Result<()> {
    let data = fs::read(input_path)?;
    if data.len() < 32 + NONCE_SIZE {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "File too short"));
    }
    
    let (key_bytes, rest) = data.split_at(32);
    let (nonce_bytes, ciphertext) = rest.split_at(NONCE_SIZE);
    
    let key = key_bytes.try_into()
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid key length"))?;
    
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(nonce_bytes);
    
    let plaintext = cipher.decrypt(nonce, ciphertext)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    
    fs::write(output_path, plaintext)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_encryption_roundtrip() {
        let test_data = b"Secret data that needs protection";
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), test_data).unwrap();
        
        encrypt_file(input_file.path(), encrypted_file.path()).unwrap();
        decrypt_file(encrypted_file.path(), decrypted_file.path()).unwrap();
        
        let result = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(test_data.as_slice(), result.as_slice());
    }
}