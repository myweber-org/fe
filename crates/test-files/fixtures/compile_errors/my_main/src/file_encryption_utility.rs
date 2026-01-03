
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

const NONCE_SIZE: usize = 12;
const SALT_SIZE: usize = 16;

pub struct FileEncryptor {
    key: Key<Aes256Gcm>,
}

impl FileEncryptor {
    pub fn from_password(password: &str, salt: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        let argon2 = Argon2::default();
        let mut key = [0u8; 32];
        
        argon2.hash_password_into(
            password.as_bytes(),
            salt,
            &mut key
        )?;
        
        Ok(Self {
            key: Key::<Aes256Gcm>::from_slice(&key).into(),
        })
    }
    
    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let mut file_data = Vec::new();
        fs::File::open(input_path)?.read_to_end(&mut file_data)?;
        
        let cipher = Aes256Gcm::new(&self.key);
        let nonce = Nonce::from_slice(&[0u8; NONCE_SIZE]);
        
        let encrypted_data = cipher.encrypt(nonce, file_data.as_ref())
            .map_err(|e| format!("Encryption failed: {}", e))?;
        
        let mut output_file = fs::File::create(output_path)?;
        output_file.write_all(&encrypted_data)?;
        
        Ok(())
    }
    
    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let mut encrypted_data = Vec::new();
        fs::File::open(input_path)?.read_to_end(&mut encrypted_data)?;
        
        let cipher = Aes256Gcm::new(&self.key);
        let nonce = Nonce::from_slice(&[0u8; NONCE_SIZE]);
        
        let decrypted_data = cipher.decrypt(nonce, encrypted_data.as_ref())
            .map_err(|e| format!("Decryption failed: {}", e))?;
        
        let mut output_file = fs::File::create(output_path)?;
        output_file.write_all(&decrypted_data)?;
        
        Ok(())
    }
}

pub fn generate_salt() -> Vec<u8> {
    let mut salt = vec![0u8; SALT_SIZE];
    OsRng.fill_bytes(&mut salt);
    salt
}

pub fn encrypt_with_password(
    password: &str,
    input_path: &Path,
    output_path: &Path
) -> Result<(), Box<dyn std::error::Error>> {
    let salt = generate_salt();
    let encryptor = FileEncryptor::from_password(password, &salt)?;
    
    let mut final_output = salt;
    let mut temp_output = Vec::new();
    
    let cipher = Aes256Gcm::new(&encryptor.key);
    let nonce = Nonce::from_slice(&[0u8; NONCE_SIZE]);
    
    let mut file_data = Vec::new();
    fs::File::open(input_path)?.read_to_end(&mut file_data)?;
    
    let encrypted_data = cipher.encrypt(nonce, file_data.as_ref())
        .map_err(|e| format!("Encryption failed: {}", e))?;
    
    temp_output.extend_from_slice(&encrypted_data);
    final_output.append(&mut temp_output);
    
    fs::write(output_path, final_output)?;
    Ok(())
}

pub fn decrypt_with_password(
    password: &str,
    input_path: &Path,
    output_path: &Path
) -> Result<(), Box<dyn std::error::Error>> {
    let encrypted_data = fs::read(input_path)?;
    
    if encrypted_data.len() < SALT_SIZE {
        return Err("Invalid encrypted file format".into());
    }
    
    let salt = &encrypted_data[..SALT_SIZE];
    let ciphertext = &encrypted_data[SALT_SIZE..];
    
    let encryptor = FileEncryptor::from_password(password, salt)?;
    let cipher = Aes256Gcm::new(&encryptor.key);
    let nonce = Nonce::from_slice(&[0u8; NONCE_SIZE]);
    
    let decrypted_data = cipher.decrypt(nonce, ciphertext)
        .map_err(|e| format!("Decryption failed: {}", e))?;
    
    fs::write(output_path, decrypted_data)?;
    Ok(())
}