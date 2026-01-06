use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use argon2::{
    password_hash::{rand_core::OsRng as ArgonRng, PasswordHasher, SaltString},
    Argon2
};
use std::fs;
use std::io::{self, Write};

const NONCE_SIZE: usize = 12;
const SALT_SIZE: usize = 16;

pub struct EncryptionResult {
    pub ciphertext: Vec<u8>,
    pub nonce: [u8; NONCE_SIZE],
    pub salt: [u8; SALT_SIZE],
}

pub fn derive_key(password: &str, salt: &[u8]) -> Result<Key<Aes256Gcm>, String> {
    let salt_string = SaltString::encode_b64(salt)
        .map_err(|e| format!("Salt encoding failed: {}", e))?;
    
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt_string)
        .map_err(|e| format!("Key derivation failed: {}", e))?;
    
    let hash_bytes = password_hash.hash.ok_or("No hash generated")?.as_bytes();
    if hash_bytes.len() < 32 {
        return Err("Derived key too short".to_string());
    }
    
    let key_slice = &hash_bytes[..32];
    Ok(*Key::<Aes256Gcm>::from_slice(key_slice))
}

pub fn encrypt_file_data(data: &[u8], password: &str) -> Result<EncryptionResult, String> {
    let mut salt = [0u8; SALT_SIZE];
    ArgonRng.fill_bytes(&mut salt);
    
    let key = derive_key(password, &salt)?;
    
    let cipher = Aes256Gcm::new(&key);
    let mut nonce_bytes = [0u8; NONCE_SIZE];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    
    let ciphertext = cipher
        .encrypt(nonce, data)
        .map_err(|e| format!("Encryption failed: {}", e))?;
    
    Ok(EncryptionResult {
        ciphertext,
        nonce: nonce_bytes,
        salt,
    })
}

pub fn decrypt_file_data(
    encrypted_data: &EncryptionResult,
    password: &str,
) -> Result<Vec<u8>, String> {
    let key = derive_key(password, &encrypted_data.salt)?;
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(&encrypted_data.nonce);
    
    cipher
        .decrypt(nonce, encrypted_data.ciphertext.as_ref())
        .map_err(|e| format!("Decryption failed: {}", e))
}

pub fn process_file_encryption(
    input_path: &str,
    output_path: &str,
    password: &str,
) -> io::Result<()> {
    let file_data = fs::read(input_path)?;
    
    let encrypted = encrypt_file_data(&file_data, password)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    
    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&encrypted.salt)?;
    output_file.write_all(&encrypted.nonce)?;
    output_file.write_all(&encrypted.ciphertext)?;
    
    Ok(())
}

pub fn process_file_decryption(
    input_path: &str,
    output_path: &str,
    password: &str,
) -> io::Result<()> {
    let encrypted_data = fs::read(input_path)?;
    
    if encrypted_data.len() < SALT_SIZE + NONCE_SIZE {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "File too short to contain encrypted data",
        ));
    }
    
    let salt = &encrypted_data[..SALT_SIZE];
    let nonce_start = SALT_SIZE;
    let nonce_end = nonce_start + NONCE_SIZE;
    let nonce = &encrypted_data[nonce_start..nonce_end];
    let ciphertext = &encrypted_data[nonce_end..];
    
    let encryption_result = EncryptionResult {
        ciphertext: ciphertext.to_vec(),
        nonce: nonce.try_into().map_err(|_| {
            io::Error::new(io::ErrorKind::InvalidData, "Invalid nonce size")
        })?,
        salt: salt.try_into().map_err(|_| {
            io::Error::new(io::ErrorKind::InvalidData, "Invalid salt size")
        })?,
    };
    
    let decrypted_data = decrypt_file_data(&encryption_result, password)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    
    fs::write(output_path, decrypted_data)?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption_cycle() {
        let test_data = b"Test data for encryption and decryption";
        let password = "secure_password_123";
        
        let encrypted = encrypt_file_data(test_data, password).unwrap();
        let decrypted = decrypt_file_data(&encrypted, password).unwrap();
        
        assert_eq!(test_data.to_vec(), decrypted);
    }

    #[test]
    fn test_file_operations() {
        let original_content = b"File content to encrypt";
        let password = "test_password";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), original_content).unwrap();
        
        process_file_encryption(
            input_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap(),
            password,
        ).unwrap();
        
        process_file_decryption(
            encrypted_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap(),
            password,
        ).unwrap();
        
        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(original_content.to_vec(), decrypted_content);
    }
}