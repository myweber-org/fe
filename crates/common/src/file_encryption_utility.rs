use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use std::error::Error;

pub struct FileEncryptor {
    cipher: Aes256Gcm,
}

impl FileEncryptor {
    pub fn new(key: &[u8; 32]) -> Result<Self, Box<dyn Error>> {
        let cipher = Aes256Gcm::new_from_slice(key)?;
        Ok(FileEncryptor { cipher })
    }

    pub fn encrypt_data(&self, plaintext: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
        let nonce = Nonce::generate(&mut OsRng);
        let ciphertext = self.cipher.encrypt(&nonce, plaintext)?;
        
        let mut result = Vec::with_capacity(nonce.len() + ciphertext.len());
        result.extend_from_slice(nonce.as_slice());
        result.extend_from_slice(&ciphertext);
        
        Ok(result)
    }

    pub fn decrypt_data(&self, encrypted_data: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
        if encrypted_data.len() < 12 {
            return Err("Invalid encrypted data length".into());
        }
        
        let nonce = Nonce::from_slice(&encrypted_data[..12]);
        let ciphertext = &encrypted_data[12..];
        
        let plaintext = self.cipher.decrypt(nonce, ciphertext)?;
        Ok(plaintext)
    }
}

pub fn generate_random_key() -> [u8; 32] {
    let mut key = [0u8; 32];
    OsRng.fill_bytes(&mut key);
    key
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_decryption() {
        let key = generate_random_key();
        let encryptor = FileEncryptor::new(&key).unwrap();
        
        let test_data = b"Secret file content that needs protection";
        
        let encrypted = encryptor.encrypt_data(test_data).unwrap();
        let decrypted = encryptor.decrypt_data(&encrypted).unwrap();
        
        assert_eq!(test_data.to_vec(), decrypted);
    }

    #[test]
    fn test_invalid_decryption() {
        let key = generate_random_key();
        let encryptor = FileEncryptor::new(&key).unwrap();
        
        let invalid_data = b"Too short";
        let result = encryptor.decrypt_data(invalid_data);
        assert!(result.is_err());
    }
}