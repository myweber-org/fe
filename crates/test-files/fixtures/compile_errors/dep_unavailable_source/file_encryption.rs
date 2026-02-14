
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use std::fs;
use std::io::{self, Write};

const NONCE_SIZE: usize = 12;

pub fn encrypt_file(input_path: &str, output_path: &str, key: &[u8; 32]) -> io::Result<()> {
    let cipher = Aes256Gcm::new_from_slice(key).unwrap();
    let nonce = Nonce::from_slice(&generate_nonce());
    
    let plaintext = fs::read(input_path)?;
    let ciphertext = cipher.encrypt(nonce, plaintext.as_ref())
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    
    let mut output = Vec::with_capacity(NONCE_SIZE + ciphertext.len());
    output.extend_from_slice(nonce);
    output.extend_from_slice(&ciphertext);
    
    fs::write(output_path, output)
}

pub fn decrypt_file(input_path: &str, output_path: &str, key: &[u8; 32]) -> io::Result<()> {
    let cipher = Aes256Gcm::new_from_slice(key).unwrap();
    let data = fs::read(input_path)?;
    
    if data.len() < NONCE_SIZE {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "File too short"));
    }
    
    let (nonce_bytes, ciphertext) = data.split_at(NONCE_SIZE);
    let nonce = Nonce::from_slice(nonce_bytes);
    
    let plaintext = cipher.decrypt(nonce, ciphertext)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    
    fs::write(output_path, plaintext)
}

fn generate_nonce() -> [u8; NONCE_SIZE] {
    let mut nonce = [0u8; NONCE_SIZE];
    OsRng.fill_bytes(&mut nonce);
    nonce
}

pub fn generate_key() -> [u8; 32] {
    let mut key = [0u8; 32];
    OsRng.fill_bytes(&mut key);
    key
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_roundtrip() {
        let key = generate_key();
        let test_data = b"Secret data that needs protection";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), test_data).unwrap();
        
        encrypt_file(
            input_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap(),
            &key
        ).unwrap();
        
        decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap(),
            &key
        ).unwrap();
        
        let decrypted = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(test_data.to_vec(), decrypted);
    }
}
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use argon2::{
    password_hash::{rand_core::OsRng as ArgonRng, PasswordHasher, SaltString},
    Argon2,
};
use std::{
    fs::{self, File},
    io::{Read, Write},
    path::Path,
};

const NONCE_SIZE: usize = 12;
const SALT_SIZE: usize = 16;

pub struct FileEncryptor {
    key: [u8; 32],
}

impl FileEncryptor {
    pub fn from_password(password: &str, salt: &[u8]) -> Result<Self, String> {
        let argon2 = Argon2::default();
        let salt_string = SaltString::encode_b64(salt).map_err(|e| e.to_string())?;
        
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt_string)
            .map_err(|e| e.to_string())?;
        
        let mut key = [0u8; 32];
        key.copy_from_slice(&password_hash.hash.unwrap().as_bytes()[..32]);
        
        Ok(Self { key })
    }
    
    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), String> {
        let mut file = File::open(input_path).map_err(|e| e.to_string())?;
        let mut plaintext = Vec::new();
        file.read_to_end(&mut plaintext).map_err(|e| e.to_string())?;
        
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&self.key));
        let nonce = Nonce::from_slice(&generate_random_bytes(NONCE_SIZE));
        
        let ciphertext = cipher
            .encrypt(nonce, plaintext.as_ref())
            .map_err(|e| e.to_string())?;
        
        let mut output = File::create(output_path).map_err(|e| e.to_string())?;
        output.write_all(nonce.as_slice()).map_err(|e| e.to_string())?;
        output.write_all(&ciphertext).map_err(|e| e.to_string())?;
        
        Ok(())
    }
    
    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), String> {
        let mut file = File::open(input_path).map_err(|e| e.to_string())?;
        let mut encrypted_data = Vec::new();
        file.read_to_end(&mut encrypted_data).map_err(|e| e.to_string())?;
        
        if encrypted_data.len() < NONCE_SIZE {
            return Err("Invalid encrypted file format".to_string());
        }
        
        let (nonce_bytes, ciphertext) = encrypted_data.split_at(NONCE_SIZE);
        let nonce = Nonce::from_slice(nonce_bytes);
        
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&self.key));
        let plaintext = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| e.to_string())?;
        
        let mut output = File::create(output_path).map_err(|e| e.to_string())?;
        output.write_all(&plaintext).map_err(|e| e.to_string())?;
        
        Ok(())
    }
    
    pub fn generate_salt() -> [u8; SALT_SIZE] {
        generate_random_bytes(SALT_SIZE).try_into().unwrap()
    }
}

fn generate_random_bytes(size: usize) -> Vec<u8> {
    let mut bytes = vec![0u8; size];
    OsRng.fill_bytes(&mut bytes);
    bytes
}

pub fn encrypt_directory(
    password: &str,
    input_dir: &Path,
    output_dir: &Path,
) -> Result<(), String> {
    let salt = FileEncryptor::generate_salt();
    let encryptor = FileEncryptor::from_password(password, &salt)?;
    
    fs::create_dir_all(output_dir).map_err(|e| e.to_string())?;
    
    let salt_file = output_dir.join("salt.bin");
    let mut salt_output = File::create(salt_file).map_err(|e| e.to_string())?;
    salt_output.write_all(&salt).map_err(|e| e.to_string())?;
    
    for entry in fs::read_dir(input_dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        
        if path.is_file() {
            let output_path = output_dir.join(
                path.file_name()
                    .ok_or_else(|| "Invalid file name".to_string())?,
            );
            
            encryptor.encrypt_file(&path, &output_path)?;
        }
    }
    
    Ok(())
}