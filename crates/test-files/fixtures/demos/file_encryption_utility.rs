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
    fs::{self, File},
    io::{Read, Write},
    path::Path
};

const NONCE_SIZE: usize = 12;
const SALT_SIZE: usize = 16;

pub struct FileEncryptor {
    cipher: Aes256Gcm,
    nonce: [u8; NONCE_SIZE],
}

impl FileEncryptor {
    pub fn from_password(password: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2.hash_password(password.as_bytes(), &salt)?;
        
        let key_bytes = password_hash.hash.ok_or("Hash generation failed")?;
        let key = Key::<Aes256Gcm>::from_slice(key_bytes.as_bytes());
        let cipher = Aes256Gcm::new(key);
        
        let mut nonce = [0u8; NONCE_SIZE];
        OsRng.fill_bytes(&mut nonce);
        
        Ok(Self {
            cipher,
            nonce,
        })
    }
    
    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let mut input_file = File::open(input_path)?;
        let mut plaintext = Vec::new();
        input_file.read_to_end(&mut plaintext)?;
        
        let ciphertext = self.cipher.encrypt(Nonce::from_slice(&self.nonce), plaintext.as_ref())
            .map_err(|e| format!("Encryption failed: {}", e))?;
        
        let mut output_file = File::create(output_path)?;
        output_file.write_all(&self.nonce)?;
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

pub fn encrypt_directory(password: &str, dir_path: &Path, output_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    if !dir_path.is_dir() {
        return Err("Provided path is not a directory".into());
    }
    
    fs::create_dir_all(output_dir)?;
    let encryptor = FileEncryptor::from_password(password)?;
    
    for entry in fs::read_dir(dir_path)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_file() {
            let output_path = output_dir.join(path.file_name().ok_or("Invalid filename")?);
            encryptor.encrypt_file(&path, &output_path)?;
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[test]
    fn test_file_encryption_decryption() {
        let temp_dir = tempdir().unwrap();
        let original_path = temp_dir.path().join("test.txt");
        let encrypted_path = temp_dir.path().join("test.enc");
        let decrypted_path = temp_dir.path().join("test_decrypted.txt");
        
        let test_content = b"Secret data that needs protection";
        fs::write(&original_path, test_content).unwrap();
        
        let encryptor = FileEncryptor::from_password("strong_password").unwrap();
        encryptor.encrypt_file(&original_path, &encrypted_path).unwrap();
        encryptor.decrypt_file(&encrypted_path, &decrypted_path).unwrap();
        
        let decrypted_content = fs::read(&decrypted_path).unwrap();
        assert_eq!(test_content.to_vec(), decrypted_content);
    }
    
    #[test]
    fn test_directory_encryption() {
        let source_dir = tempdir().unwrap();
        let output_dir = tempdir().unwrap();
        
        let file1_path = source_dir.path().join("file1.txt");
        let file2_path = source_dir.path().join("file2.txt");
        
        fs::write(&file1_path, b"First secret file").unwrap();
        fs::write(&file2_path, b"Second secret file").unwrap();
        
        encrypt_directory("directory_password", source_dir.path(), output_dir.path()).unwrap();
        
        let encrypted_files: Vec<_> = fs::read_dir(output_dir.path()).unwrap()
            .map(|entry| entry.unwrap().path())
            .collect();
        
        assert_eq!(encrypted_files.len(), 2);
    }
}