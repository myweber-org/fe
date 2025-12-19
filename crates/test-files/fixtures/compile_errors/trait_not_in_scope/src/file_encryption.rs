
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use pbkdf2::{
    password_hash::{
        PasswordHasher, SaltString, PasswordHash, PasswordVerifier
    },
    Pbkdf2
};
use std::fs;
use std::io::{Read, Write};

const SALT_LENGTH: usize = 16;
const NONCE_LENGTH: usize = 12;

pub struct FileEncryptor {
    cipher: Aes256Gcm,
}

impl FileEncryptor {
    pub fn new(password: &str) -> Result<Self, String> {
        let salt = SaltString::generate(&mut OsRng);
        let password_hash = Pbkdf2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| format!("Key derivation failed: {}", e))?;
        
        let hash_bytes = password_hash.hash.ok_or("No hash generated")?.as_bytes();
        if hash_bytes.len() < 32 {
            return Err("Derived key too short".to_string());
        }
        
        let key_bytes = &hash_bytes[..32];
        let key = Key::<Aes256Gcm>::from_slice(key_bytes);
        let cipher = Aes256Gcm::new(key);
        
        Ok(Self { cipher })
    }
    
    pub fn encrypt_file(&self, input_path: &str, output_path: &str) -> Result<(), String> {
        let mut file = fs::File::open(input_path)
            .map_err(|e| format!("Failed to open input file: {}", e))?;
        
        let mut plaintext = Vec::new();
        file.read_to_end(&mut plaintext)
            .map_err(|e| format!("Failed to read file: {}", e))?;
        
        let nonce = Nonce::generate(&mut OsRng);
        let ciphertext = self.cipher
            .encrypt(&nonce, plaintext.as_ref())
            .map_err(|e| format!("Encryption failed: {}", e))?;
        
        let mut output = fs::File::create(output_path)
            .map_err(|e| format!("Failed to create output file: {}", e))?;
        
        output.write_all(nonce.as_slice())
            .map_err(|e| format!("Failed to write nonce: {}", e))?;
        output.write_all(&ciphertext)
            .map_err(|e| format!("Failed to write ciphertext: {}", e))?;
        
        Ok(())
    }
    
    pub fn decrypt_file(&self, input_path: &str, output_path: &str) -> Result<(), String> {
        let mut file = fs::File::open(input_path)
            .map_err(|e| format!("Failed to open input file: {}", e))?;
        
        let mut encrypted_data = Vec::new();
        file.read_to_end(&mut encrypted_data)
            .map_err(|e| format!("Failed to read file: {}", e))?;
        
        if encrypted_data.len() < NONCE_LENGTH {
            return Err("File too short to contain nonce".to_string());
        }
        
        let (nonce_bytes, ciphertext) = encrypted_data.split_at(NONCE_LENGTH);
        let nonce = Nonce::from_slice(nonce_bytes);
        
        let plaintext = self.cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| format!("Decryption failed: {}", e))?;
        
        let mut output = fs::File::create(output_path)
            .map_err(|e| format!("Failed to create output file: {}", e))?;
        output.write_all(&plaintext)
            .map_err(|e| format!("Failed to write plaintext: {}", e))?;
        
        Ok(())
    }
}

pub fn verify_password(password: &str, stored_hash: &str) -> Result<bool, String> {
    let parsed_hash = PasswordHash::new(stored_hash)
        .map_err(|e| format!("Invalid hash format: {}", e))?;
    
    Ok(Pbkdf2.verify_password(password.as_bytes(), &parsed_hash).is_ok())
}