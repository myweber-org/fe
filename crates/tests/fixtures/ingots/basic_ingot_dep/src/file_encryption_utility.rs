use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use std::fs;
use std::io::{self, Write};
use std::path::Path;

pub struct FileEncryptor {
    cipher: Aes256Gcm,
}

impl FileEncryptor {
    pub fn new() -> Self {
        let key = Key::<Aes256Gcm>::generate(&mut OsRng);
        Self {
            cipher: Aes256Gcm::new(&key),
        }
    }

    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> io::Result<()> {
        let plaintext = fs::read(input_path)?;
        let nonce = Nonce::generate(&mut OsRng);
        
        let ciphertext = self.cipher
            .encrypt(&nonce, plaintext.as_ref())
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        
        let mut output_data = nonce.to_vec();
        output_data.extend_from_slice(&ciphertext);
        
        fs::write(output_path, output_data)
    }

    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path) -> io::Result<()> {
        let data = fs::read(input_path)?;
        
        if data.len() < 12 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "File too short to contain valid encrypted data"
            ));
        }
        
        let (nonce_bytes, ciphertext) = data.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);
        
        let plaintext = self.cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        
        fs::write(output_path, plaintext)
    }
}

pub fn generate_key() -> Vec<u8> {
    let key = Key::<Aes256Gcm>::generate(&mut OsRng);
    key.to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let encryptor = FileEncryptor::new();
        let test_data = b"Test encryption data";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), test_data).unwrap();
        
        encryptor.encrypt_file(input_file.path(), encrypted_file.path())
            .expect("Encryption failed");
        
        encryptor.decrypt_file(encrypted_file.path(), decrypted_file.path())
            .expect("Decryption failed");
        
        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(test_data.to_vec(), decrypted_data);
    }
}use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use argon2::{
    password_hash::{rand_core::OsRng as ArgonRng, PasswordHasher, SaltString},
    Argon2,
};
use std::fs;
use std::io::{self, Read, Write};

const NONCE_SIZE: usize = 12;
const SALT_SIZE: usize = 16;

pub fn encrypt_file(input_path: &str, output_path: &str, password: &str) -> io::Result<()> {
    let mut file_data = Vec::new();
    fs::File::open(input_path)?.read_to_end(&mut file_data)?;

    let salt = SaltString::generate(&mut ArgonRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

    let key_bytes = password_hash.hash.unwrap().as_bytes();
    let key = Key::<Aes256Gcm>::from_slice(&key_bytes[..32]);
    let cipher = Aes256Gcm::new(key);

    let mut nonce_bytes = [0u8; NONCE_SIZE];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, file_data.as_ref())
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

    let mut output_data = Vec::new();
    output_data.extend_from_slice(salt.as_bytes());
    output_data.extend_from_slice(&nonce_bytes);
    output_data.extend_from_slice(&ciphertext);

    fs::File::create(output_path)?.write_all(&output_data)?;
    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str, password: &str) -> io::Result<()> {
    let mut encrypted_data = Vec::new();
    fs::File::open(input_path)?.read_to_end(&mut encrypted_data)?;

    if encrypted_data.len() < SALT_SIZE + NONCE_SIZE {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "File too short to contain salt and nonce",
        ));
    }

    let salt = SaltString::from_b64(
        std::str::from_utf8(&encrypted_data[..SALT_SIZE])
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?,
    )
    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

    let key_bytes = password_hash.hash.unwrap().as_bytes();
    let key = Key::<Aes256Gcm>::from_slice(&key_bytes[..32]);
    let cipher = Aes256Gcm::new(key);

    let nonce_start = SALT_SIZE;
    let nonce_end = nonce_start + NONCE_SIZE;
    let ciphertext_start = nonce_end;

    let nonce = Nonce::from_slice(&encrypted_data[nonce_start..nonce_end]);
    let ciphertext = &encrypted_data[ciphertext_start..];

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Decryption failed"))?;

    fs::File::create(output_path)?.write_all(&plaintext)?;
    Ok(())
}