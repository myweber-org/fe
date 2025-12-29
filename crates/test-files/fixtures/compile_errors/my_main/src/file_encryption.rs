
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use std::fs;
use std::io::{self, Write};
use std::path::Path;

pub struct FileEncryptor {
    cipher: Aes256Gcm,
}

impl FileEncryptor {
    pub fn new() -> Self {
        let key = Aes256Gcm::generate_key(&mut OsRng);
        let cipher = Aes256Gcm::new(&key);
        FileEncryptor { cipher }
    }

    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> io::Result<()> {
        let plaintext = fs::read(input_path)?;
        let nonce = Nonce::from_slice(b"unique_nonce_");
        
        let ciphertext = self.cipher
            .encrypt(nonce, plaintext.as_ref())
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        
        fs::write(output_path, ciphertext)
    }

    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path) -> io::Result<()> {
        let ciphertext = fs::read(input_path)?;
        let nonce = Nonce::from_slice(b"unique_nonce_");
        
        let plaintext = self.cipher
            .decrypt(nonce, ciphertext.as_ref())
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        
        fs::write(output_path, plaintext)
    }
}

pub fn generate_secure_random_bytes(count: usize) -> Vec<u8> {
    let mut bytes = vec![0u8; count];
    OsRng.fill_bytes(&mut bytes);
    bytes
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_roundtrip() {
        let encryptor = FileEncryptor::new();
        let test_data = b"Secret data that needs protection";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), test_data).unwrap();
        
        encryptor.encrypt_file(input_file.path(), encrypted_file.path()).unwrap();
        encryptor.decrypt_file(encrypted_file.path(), decrypted_file.path()).unwrap();
        
        let result = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(test_data.to_vec(), result);
    }
}