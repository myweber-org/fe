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
use std::path::Path;

pub struct EncryptionResult {
    pub ciphertext: Vec<u8>,
    pub nonce: Vec<u8>,
    pub salt: String,
}

pub fn derive_key(password: &str, salt: &str) -> Vec<u8> {
    let salt_bytes = SaltString::from_b64(salt).unwrap();
    let argon2 = Argon2::default();
    let password_hash = argon2.hash_password(password.as_bytes(), &salt_bytes).unwrap();
    password_hash.hash.unwrap().as_bytes().to_vec()
}

pub fn encrypt_file(
    file_path: &Path,
    password: &str
) -> Result<EncryptionResult, Box<dyn std::error::Error>> {
    let plaintext = fs::read(file_path)?;
    
    let salt = SaltString::generate(&mut OsRng);
    let key_material = derive_key(password, salt.as_str());
    
    let key = Key::<Aes256Gcm>::from_slice(&key_material);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::generate(&mut OsRng);
    
    let ciphertext = cipher
        .encrypt(&nonce, plaintext.as_ref())
        .map_err(|e| format!("Encryption failed: {}", e))?;
    
    Ok(EncryptionResult {
        ciphertext,
        nonce: nonce.to_vec(),
        salt: salt.to_string(),
    })
}

pub fn decrypt_file(
    encrypted_data: &EncryptionResult,
    password: &str
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let key_material = derive_key(password, &encrypted_data.salt);
    let key = Key::<Aes256Gcm>::from_slice(&key_material);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(&encrypted_data.nonce);
    
    let plaintext = cipher
        .decrypt(nonce, encrypted_data.ciphertext.as_ref())
        .map_err(|e| format!("Decryption failed: {}", e))?;
    
    Ok(plaintext)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_roundtrip() {
        let test_content = b"Secret data that needs protection";
        let password = "strong_password_123";
        
        let mut temp_file = NamedTempFile::new().unwrap();
        std::fs::write(temp_file.path(), test_content).unwrap();
        
        let encrypted = encrypt_file(temp_file.path(), password).unwrap();
        let decrypted = decrypt_file(&encrypted, password).unwrap();
        
        assert_eq!(test_content.to_vec(), decrypted);
    }
    
    #[test]
    fn test_wrong_password_fails() {
        let test_content = b"Test data";
        let password = "correct_password";
        let wrong_password = "wrong_password";
        
        let mut temp_file = NamedTempFile::new().unwrap();
        std::fs::write(temp_file.path(), test_content).unwrap();
        
        let encrypted = encrypt_file(temp_file.path(), password).unwrap();
        let result = decrypt_file(&encrypted, wrong_password);
        
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
use std::{
    fs,
    io::{self, Read, Write},
    path::Path
};

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
    password: &str
) -> Result<EncryptionResult, Box<dyn std::error::Error>> {
    let mut file_data = Vec::new();
    fs::File::open(input_path)?.read_to_end(&mut file_data)?;
    
    let mut rng = OsRng;
    let mut salt = [0u8; SALT_SIZE];
    rng.fill_bytes(&mut salt);
    
    let mut nonce = [0u8; NONCE_SIZE];
    rng.fill_bytes(&mut nonce);
    
    let key_bytes = derive_key(password, &salt)?;
    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);
    
    let ciphertext = cipher
        .encrypt(Nonce::from_slice(&nonce), file_data.as_ref())
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    
    let result = EncryptionResult {
        ciphertext: ciphertext.clone(),
        nonce,
        salt,
    };
    
    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&salt)?;
    output_file.write_all(&nonce)?;
    output_file.write_all(&ciphertext)?;
    
    Ok(result)
}

pub fn decrypt_file(
    input_path: &Path,
    output_path: &Path,
    password: &str
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut encrypted_data = Vec::new();
    fs::File::open(input_path)?.read_to_end(&mut encrypted_data)?;
    
    if encrypted_data.len() < SALT_SIZE + NONCE_SIZE {
        return Err("Invalid encrypted file format".into());
    }
    
    let salt = &encrypted_data[..SALT_SIZE];
    let nonce = &encrypted_data[SALT_SIZE..SALT_SIZE + NONCE_SIZE];
    let ciphertext = &encrypted_data[SALT_SIZE + NONCE_SIZE..];
    
    let key_bytes = derive_key(password, salt)?;
    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);
    
    let plaintext = cipher
        .decrypt(Nonce::from_slice(nonce), ciphertext)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    
    fs::File::create(output_path)?.write_all(&plaintext)?;
    
    Ok(plaintext)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_encryption_decryption() {
        let test_data = b"Test encryption data for AES-256-GCM";
        let password = "secure_password_123";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), test_data).unwrap();
        
        encrypt_file(input_file.path(), encrypted_file.path(), password)
            .expect("Encryption failed");
        
        decrypt_file(encrypted_file.path(), decrypted_file.path(), password)
            .expect("Decryption failed");
        
        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(test_data.to_vec(), decrypted_data);
    }
    
    #[test]
    fn test_wrong_password_fails() {
        let test_data = b"Sensitive information";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), test_data).unwrap();
        
        encrypt_file(input_file.path(), encrypted_file.path(), "correct_password")
            .expect("Encryption failed");
        
        let result = decrypt_file(encrypted_file.path(), decrypted_file.path(), "wrong_password");
        assert!(result.is_err());
    }
}