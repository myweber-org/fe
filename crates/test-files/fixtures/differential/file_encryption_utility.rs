
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use pbkdf2::{
    password_hash::{PasswordHasher, SaltString},
    Pbkdf2,
};
use std::{
    fs::{self, File},
    io::{Read, Write},
    path::Path,
};

const SALT_LENGTH: usize = 16;
const NONCE_LENGTH: usize = 12;
const PBKDF2_ITERATIONS: u32 = 100_000;

pub struct EncryptionResult {
    pub encrypted_data: Vec<u8>,
    pub salt: [u8; SALT_LENGTH],
    pub nonce: [u8; NONCE_LENGTH],
}

pub fn derive_key(password: &str, salt: &[u8]) -> Result<Key<Aes256Gcm>, String> {
    let salt_string = SaltString::encode_b64(salt).map_err(|e| e.to_string())?;
    let password_hash = Pbkdf2
        .hash_password(password.as_bytes(), &salt_string)
        .map_err(|e| e.to_string())?;
    
    let hash_bytes = password_hash.hash.ok_or("Hash generation failed")?;
    let key_bytes: [u8; 32] = hash_bytes.as_bytes()[..32]
        .try_into()
        .map_err(|_| "Key derivation failed")?;
    
    Ok(Key::<Aes256Gcm>::from_slice(&key_bytes).clone())
}

pub fn encrypt_data(data: &[u8], password: &str) -> Result<EncryptionResult, String> {
    let mut salt = [0u8; SALT_LENGTH];
    OsRng.fill_bytes(&mut salt);
    
    let mut nonce = [0u8; NONCE_LENGTH];
    OsRng.fill_bytes(&mut nonce);
    
    let key = derive_key(password, &salt)?;
    let cipher = Aes256Gcm::new(&key);
    
    let encrypted_data = cipher
        .encrypt(Nonce::from_slice(&nonce), data)
        .map_err(|e| e.to_string())?;
    
    Ok(EncryptionResult {
        encrypted_data,
        salt,
        nonce,
    })
}

pub fn decrypt_data(
    encrypted_data: &[u8],
    password: &str,
    salt: &[u8],
    nonce: &[u8],
) -> Result<Vec<u8>, String> {
    let key = derive_key(password, salt)?;
    let cipher = Aes256Gcm::new(&key);
    
    cipher
        .decrypt(Nonce::from_slice(nonce), encrypted_data)
        .map_err(|e| e.to_string())
}

pub fn encrypt_file(
    input_path: &Path,
    output_path: &Path,
    password: &str,
) -> Result<EncryptionResult, String> {
    let mut file = File::open(input_path).map_err(|e| e.to_string())?;
    let mut data = Vec::new();
    file.read_to_end(&mut data).map_err(|e| e.to_string())?;
    
    let result = encrypt_data(&data, password)?;
    
    let mut output_file = File::create(output_path).map_err(|e| e.to_string())?;
    output_file
        .write_all(&result.encrypted_data)
        .map_err(|e| e.to_string())?;
    
    let metadata_path = output_path.with_extension("meta");
    let mut meta_file = File::create(metadata_path).map_err(|e| e.to_string())?;
    meta_file
        .write_all(&result.salt)
        .map_err(|e| e.to_string())?;
    meta_file
        .write_all(&result.nonce)
        .map_err(|e| e.to_string())?;
    
    Ok(result)
}

pub fn decrypt_file(
    input_path: &Path,
    output_path: &Path,
    password: &str,
) -> Result<Vec<u8>, String> {
    let mut encrypted_file = File::open(input_path).map_err(|e| e.to_string())?;
    let mut encrypted_data = Vec::new();
    encrypted_file
        .read_to_end(&mut encrypted_data)
        .map_err(|e| e.to_string())?;
    
    let metadata_path = input_path.with_extension("meta");
    let mut meta_file = File::open(metadata_path).map_err(|e| e.to_string())?;
    
    let mut salt = [0u8; SALT_LENGTH];
    meta_file.read_exact(&mut salt).map_err(|e| e.to_string())?;
    
    let mut nonce = [0u8; NONCE_LENGTH];
    meta_file.read_exact(&mut nonce).map_err(|e| e.to_string())?;
    
    let decrypted_data = decrypt_data(&encrypted_data, password, &salt, &nonce)?;
    
    let mut output_file = File::create(output_path).map_err(|e| e.to_string())?;
    output_file
        .write_all(&decrypted_data)
        .map_err(|e| e.to_string())?;
    
    Ok(decrypted_data)
}

pub fn generate_secure_password(length: usize) -> Result<String, String> {
    if length < 12 {
        return Err("Password length must be at least 12 characters".to_string());
    }
    
    let charset: Vec<char> = "ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                             abcdefghijklmnopqrstuvwxyz\
                             0123456789\
                             !@#$%^&*()-_=+[]{}|;:,.<>?"
        .chars()
        .collect();
    
    let mut password = String::with_capacity(length);
    let mut rng = OsRng;
    
    for _ in 0..length {
        let idx = (rng.next_u32() as usize) % charset.len();
        password.push(charset[idx]);
    }
    
    Ok(password)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let test_data = b"Secret data that needs protection";
        let password = "StrongPassword123!";
        
        let encrypted = encrypt_data(test_data, password).unwrap();
        let decrypted = decrypt_data(
            &encrypted.encrypted_data,
            password,
            &encrypted.salt,
            &encrypted.nonce,
        ).unwrap();
        
        assert_eq!(test_data.to_vec(), decrypted);
    }

    #[test]
    fn test_file_encryption() {
        let input_file = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();
        let password = "TestPassword456!";
        
        let test_content = b"File content to encrypt";
        fs::write(input_file.path(), test_content).unwrap();
        
        let _ = encrypt_file(input_file.path(), output_file.path(), password).unwrap();
        
        let decrypted_path = NamedTempFile::new().unwrap().into_temp_path();
        let decrypted = decrypt_file(output_file.path(), &decrypted_path, password).unwrap();
        
        assert_eq!(test_content.to_vec(), decrypted);
    }

    #[test]
    fn test_password_generation() {
        let password = generate_secure_password(16).unwrap();
        assert_eq!(password.len(), 16);
        assert!(password.chars().any(|c| c.is_ascii_uppercase()));
        assert!(password.chars().any(|c| c.is_ascii_lowercase()));
        assert!(password.chars().any(|c| c.is_ascii_digit()));
        assert!(password.chars().any(|c| "!@#$%^&*()-_=+[]{}|;:,.<>?".contains(c)));
    }
}