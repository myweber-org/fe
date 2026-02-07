use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use argon2::{
    password_hash::{rand_core::OsRng as ArgonRng, PasswordHasher, SaltString},
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
        let salt_string = SaltString::encode_b64(salt)?;
        let argon2 = Argon2::default();
        let password_hash = argon2.hash_password(password.as_bytes(), &salt_string)?;
        
        let key_bytes = password_hash.hash.ok_or("Failed to derive key")?.as_bytes();
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
    ArgonRng.fill_bytes(&mut salt);
    salt
}

pub fn process_directory(
    encryptor: &FileEncryptor,
    dir_path: &Path,
    operation: fn(&FileEncryptor, &Path, &Path) -> Result<(), Box<dyn std::error::Error>>,
    suffix: &str
) -> Result<(), Box<dyn std::error::Error>> {
    for entry in fs::read_dir(dir_path)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_file() {
            let output_path = path.with_extension(format!("{}.{}", path.extension().unwrap_or_default().to_str().unwrap_or(""), suffix));
            operation(encryptor, &path, &output_path)?;
            println!("Processed: {:?} -> {:?}", path, output_path);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_encryption_decryption() {
        let salt = generate_salt();
        let encryptor = FileEncryptor::from_password("test_password", &salt).unwrap();
        
        let mut temp_input = NamedTempFile::new().unwrap();
        let original_data = b"Secret data for encryption test";
        temp_input.write_all(original_data).unwrap();
        
        let temp_encrypted = NamedTempFile::new().unwrap();
        let temp_decrypted = NamedTempFile::new().unwrap();
        
        encryptor.encrypt_file(temp_input.path(), temp_encrypted.path()).unwrap();
        encryptor.decrypt_file(temp_encrypted.path(), temp_decrypted.path()).unwrap();
        
        let mut decrypted_data = Vec::new();
        File::open(temp_decrypted.path()).unwrap().read_to_end(&mut decrypted_data).unwrap();
        
        assert_eq!(decrypted_data, original_data);
    }
}