use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use std::error::Error;

pub fn encrypt_data(plaintext: &[u8], key: &[u8; 32]) -> Result<Vec<u8>, Box<dyn Error>> {
    let cipher = Aes256Gcm::new_from_slice(key)?;
    let nonce = Nonce::from_slice(&[0u8; 12]);
    
    let ciphertext = cipher.encrypt(nonce, plaintext)?;
    Ok(ciphertext)
}

pub fn decrypt_data(ciphertext: &[u8], key: &[u8; 32]) -> Result<Vec<u8>, Box<dyn Error>> {
    let cipher = Aes256Gcm::new_from_slice(key)?;
    let nonce = Nonce::from_slice(&[0u8; 12]);
    
    let plaintext = cipher.decrypt(nonce, ciphertext)?;
    Ok(plaintext)
}

pub fn generate_key() -> [u8; 32] {
    let mut key = [0u8; 32];
    OsRng.fill_bytes(&mut key);
    key
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_roundtrip() {
        let key = generate_key();
        let original_data = b"Secret message for encryption test";
        
        let encrypted = encrypt_data(original_data, &key).unwrap();
        let decrypted = decrypt_data(&encrypted, &key).unwrap();
        
        assert_eq!(original_data.to_vec(), decrypted);
    }

    #[test]
    fn test_wrong_key_fails() {
        let key1 = generate_key();
        let key2 = generate_key();
        let data = b"Test data";
        
        let encrypted = encrypt_data(data, &key1).unwrap();
        let result = decrypt_data(&encrypted, &key2);
        
        assert!(result.is_err());
    }
}