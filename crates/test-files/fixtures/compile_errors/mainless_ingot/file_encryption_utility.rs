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
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;

const NONCE_SIZE: usize = 12;
const SALT_SIZE: usize = 16;

pub struct FileEncryptor {
    cipher: Aes256Gcm,
}

impl FileEncryptor {
    pub fn from_password(password: &str, salt: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        let argon2 = Argon2::default();
        let salt_string = SaltString::encode_b64(salt)?;
        
        let password_hash = argon2.hash_password(password.as_bytes(), &salt_string)?;
        let key_bytes = password_hash.hash.ok_or("Failed to generate key")?.as_bytes();
        
        let key = Key::<Aes256Gcm>::from_slice(key_bytes);
        let cipher = Aes256Gcm::new(key);
        
        Ok(FileEncryptor { cipher })
    }
    
    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let mut input_file = File::open(input_path)?;
        let mut plaintext = Vec::new();
        input_file.read_to_end(&mut plaintext)?;
        
        let mut nonce = [0u8; NONCE_SIZE];
        OsRng.fill_bytes(&mut nonce);
        
        let ciphertext = self.cipher.encrypt(Nonce::from_slice(&nonce), plaintext.as_ref())
            .map_err(|e| format!("Encryption failed: {}", e))?;
        
        let mut output_file = File::create(output_path)?;
        output_file.write_all(&nonce)?;
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

pub fn generate_salt() -> Vec<u8> {
    let mut salt = vec![0u8; SALT_SIZE];
    OsRng.fill_bytes(&mut salt);
    salt
}

pub fn process_directory(
    encryptor: &FileEncryptor,
    dir_path: &Path,
    output_dir: &Path,
    encrypt: bool
) -> Result<(), Box<dyn std::error::Error>> {
    if !output_dir.exists() {
        fs::create_dir_all(output_dir)?;
    }
    
    for entry in fs::read_dir(dir_path)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_file() {
            let file_name = path.file_name().ok_or("Invalid file name")?;
            let output_path = output_dir.join(file_name);
            
            if encrypt {
                encryptor.encrypt_file(&path, &output_path)?;
                println!("Encrypted: {:?} -> {:?}", path, output_path);
            } else {
                encryptor.decrypt_file(&path, &output_path)?;
                println!("Decrypted: {:?} -> {:?}", path, output_path);
            }
        }
    }
    
    Ok(())
}