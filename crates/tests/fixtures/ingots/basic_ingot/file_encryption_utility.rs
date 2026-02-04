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

const NONCE_SIZE: usize = 12;
const SALT_SIZE: usize = 16;

pub struct EncryptionResult {
    pub ciphertext: Vec<u8>,
    pub nonce: [u8; NONCE_SIZE],
    pub salt: [u8; SALT_SIZE],
}

pub fn derive_key(password: &str, salt: &[u8]) -> Result<Key<Aes256Gcm>, String> {
    let salt_string = SaltString::encode_b64(salt).map_err(|e| e.to_string())?;
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt_string)
        .map_err(|e| e.to_string())?;

    let hash_bytes = password_hash.hash.ok_or("Hash generation failed")?;
    let key_bytes: [u8; 32] = hash_bytes.as_bytes()[..32]
        .try_into()
        .map_err(|_| "Hash too short")?;

    Ok(Key::<Aes256Gcm>::from_slice(&key_bytes).clone())
}

pub fn encrypt_file(
    input_path: &Path,
    output_path: &Path,
    password: &str,
) -> Result<EncryptionResult, String> {
    let mut file = File::open(input_path).map_err(|e| e.to_string())?;
    let mut plaintext = Vec::new();
    file.read_to_end(&mut plaintext).map_err(|e| e.to_string())?;

    let mut rng = OsRng;
    let mut salt = [0u8; SALT_SIZE];
    rng.fill_bytes(&mut salt);
    let mut nonce_bytes = [0u8; NONCE_SIZE];
    rng.fill_bytes(&mut nonce_bytes);

    let key = derive_key(password, &salt)?;
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_ref())
        .map_err(|e| e.to_string())?;

    let mut output_file = File::create(output_path).map_err(|e| e.to_string())?;
    output_file.write_all(&ciphertext).map_err(|e| e.to_string())?;

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
    nonce: &[u8; NONCE_SIZE],
    salt: &[u8; SALT_SIZE],
) -> Result<(), String> {
    let mut file = File::open(input_path).map_err(|e| e.to_string())?;
    let mut ciphertext = Vec::new();
    file.read_to_end(&mut ciphertext).map_err(|e| e.to_string())?;

    let key = derive_key(password, salt)?;
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(nonce);

    let plaintext = cipher
        .decrypt(nonce, ciphertext.as_ref())
        .map_err(|e| e.to_string())?;

    let mut output_file = File::create(output_path).map_err(|e| e.to_string())?;
    output_file.write_all(&plaintext).map_err(|e| e.to_string())?;

    Ok(())
}

pub fn generate_random_key() -> Result<[u8; 32], String> {
    let mut rng = OsRng;
    let mut key = [0u8; 32];
    rng.fill_bytes(&mut key);
    Ok(key)
}

pub fn save_key_to_file(key: &[u8], path: &Path) -> Result<(), String> {
    let encoded = hex::encode(key);
    fs::write(path, encoded).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn load_key_from_file(path: &Path) -> Result<[u8; 32], String> {
    let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let decoded = hex::decode(content.trim()).map_err(|e| e.to_string())?;
    decoded.try_into().map_err(|_| "Invalid key length".to_string())
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
const SALT_SIZE: usize = 16;

pub struct FileEncryptor {
    key: [u8; 32],
}

impl FileEncryptor {
    pub fn from_password(password: &str, salt: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        let argon2 = Argon2::default();
        let salt_string = SaltString::encode_b64(salt)?;
        
        let password_hash = argon2.hash_password(password.as_bytes(), &salt_string)?;
        let hash_bytes = password_hash.hash.ok_or("Hash generation failed")?;
        
        let mut key = [0u8; 32];
        key.copy_from_slice(&hash_bytes.as_bytes()[..32]);
        
        Ok(Self { key })
    }
    
    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let mut file_data = Vec::new();
        fs::File::open(input_path)?.read_to_end(&mut file_data)?;
        
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&self.key));
        let nonce = Nonce::from_slice(&generate_random_bytes(NONCE_SIZE));
        
        let encrypted_data = cipher.encrypt(nonce, file_data.as_ref())
            .map_err(|e| format!("Encryption failed: {}", e))?;
        
        let mut output = fs::File::create(output_path)?;
        output.write_all(nonce.as_slice())?;
        output.write_all(&encrypted_data)?;
        
        Ok(())
    }
    
    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let mut encrypted_data = Vec::new();
        fs::File::open(input_path)?.read_to_end(&mut encrypted_data)?;
        
        if encrypted_data.len() < NONCE_SIZE {
            return Err("Invalid encrypted file format".into());
        }
        
        let (nonce_bytes, ciphertext) = encrypted_data.split_at(NONCE_SIZE);
        let nonce = Nonce::from_slice(nonce_bytes);
        
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&self.key));
        let decrypted_data = cipher.decrypt(nonce, ciphertext)
            .map_err(|e| format!("Decryption failed: {}", e))?;
        
        fs::write(output_path, decrypted_data)?;
        
        Ok(())
    }
    
    pub fn generate_salt() -> [u8; SALT_SIZE] {
        generate_random_bytes(SALT_SIZE)
    }
}

fn generate_random_bytes(size: usize) -> [u8; SALT_SIZE] {
    let mut bytes = [0u8; SALT_SIZE];
    OsRng.fill_bytes(&mut bytes);
    bytes
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_encryption_decryption() {
        let password = "secure_password_123";
        let salt = FileEncryptor::generate_salt();
        
        let encryptor = FileEncryptor::from_password(password, &salt).unwrap();
        
        let original_content = b"Secret data that needs protection";
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), original_content).unwrap();
        
        encryptor.encrypt_file(input_file.path(), encrypted_file.path()).unwrap();
        encryptor.decrypt_file(encrypted_file.path(), decrypted_file.path()).unwrap();
        
        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(original_content.to_vec(), decrypted_content);
    }
}