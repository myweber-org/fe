
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

const SALT_LENGTH: usize = 16;
const NONCE_LENGTH: usize = 12;

pub struct FileEncryptor {
    cipher: Aes256Gcm,
}

impl FileEncryptor {
    pub fn from_password(password: &str) -> io::Result<Self> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
        
        let key_bytes = password_hash.hash.ok_or_else(|| {
            io::Error::new(io::ErrorKind::Other, "Failed to derive key from password")
        })?.as_bytes();
        
        if key_bytes.len() != 32 {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Invalid key length: {}", key_bytes.len())
            ));
        }
        
        let key = Key::<Aes256Gcm>::from_slice(key_bytes);
        let cipher = Aes256Gcm::new(key);
        
        Ok(Self { cipher })
    }
    
    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> io::Result<()> {
        let mut input_file = fs::File::open(input_path)?;
        let mut plaintext = Vec::new();
        input_file.read_to_end(&mut plaintext)?;
        
        let mut rng = OsRng;
        let nonce_bytes: [u8; NONCE_LENGTH] = rng.gen();
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        let ciphertext = self.cipher
            .encrypt(nonce, plaintext.as_ref())
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
        
        let mut output_file = fs::File::create(output_path)?;
        output_file.write_all(&nonce_bytes)?;
        output_file.write_all(&ciphertext)?;
        
        Ok(())
    }
    
    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path) -> io::Result<()> {
        let mut input_file = fs::File::open(input_path)?;
        let mut encrypted_data = Vec::new();
        input_file.read_to_end(&mut encrypted_data)?;
        
        if encrypted_data.len() < NONCE_LENGTH {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "File too short to contain nonce"
            ));
        }
        
        let (nonce_bytes, ciphertext) = encrypted_data.split_at(NONCE_LENGTH);
        let nonce = Nonce::from_slice(nonce_bytes);
        
        let plaintext = self.cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
        
        let mut output_file = fs::File::create(output_path)?;
        output_file.write_all(&plaintext)?;
        
        Ok(())
    }
}

pub fn encrypt_directory(password: &str, dir_path: &Path, output_dir: &Path) -> io::Result<()> {
    let encryptor = FileEncryptor::from_password(password)?;
    
    if !output_dir.exists() {
        fs::create_dir_all(output_dir)?;
    }
    
    for entry in fs::read_dir(dir_path)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_file() {
            let file_name = path.file_name()
                .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Invalid filename"))?
                .to_string_lossy();
            
            let output_path = output_dir.join(format!("{}.enc", file_name));
            encryptor.encrypt_file(&path, &output_path)?;
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_encryption_decryption() {
        let temp_dir = TempDir::new().unwrap();
        let original_path = temp_dir.path().join("test.txt");
        let encrypted_path = temp_dir.path().join("test.enc");
        let decrypted_path = temp_dir.path().join("test_decrypted.txt");
        
        let test_data = b"Hello, this is a secret message!";
        fs::write(&original_path, test_data).unwrap();
        
        let encryptor = FileEncryptor::from_password("strong_password").unwrap();
        encryptor.encrypt_file(&original_path, &encrypted_path).unwrap();
        encryptor.decrypt_file(&encrypted_path, &decrypted_path).unwrap();
        
        let decrypted_data = fs::read(&decrypted_path).unwrap();
        assert_eq!(test_data.to_vec(), decrypted_data);
    }
}