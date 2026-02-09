use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use anyhow::{Context, Result};

pub fn encrypt_data(plaintext: &[u8], key: &[u8]) -> Result<Vec<u8>> {
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let nonce = Nonce::generate(&mut OsRng);
    
    let ciphertext = cipher
        .encrypt(&nonce, plaintext)
        .context("Encryption failed")?;
    
    let mut result = Vec::with_capacity(nonce.len() + ciphertext.len());
    result.extend_from_slice(nonce.as_slice());
    result.extend_from_slice(&ciphertext);
    
    Ok(result)
}

pub fn decrypt_data(ciphertext: &[u8], key: &[u8]) -> Result<Vec<u8>> {
    if ciphertext.len() < 12 {
        anyhow::bail!("Ciphertext too short");
    }
    
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let (nonce_slice, encrypted_data) = ciphertext.split_at(12);
    let nonce = Nonce::from_slice(nonce_slice);
    
    let plaintext = cipher
        .decrypt(nonce, encrypted_data)
        .context("Decryption failed")?;
    
    Ok(plaintext)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::RngCore;

    #[test]
    fn test_encryption_roundtrip() {
        let mut key = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut key);
        
        let plaintext = b"Secret message for testing";
        
        let encrypted = encrypt_data(plaintext, &key).unwrap();
        let decrypted = decrypt_data(&encrypted, &key).unwrap();
        
        assert_eq!(plaintext.to_vec(), decrypted);
    }

    #[test]
    fn test_wrong_key_fails() {
        let mut key1 = [0u8; 32];
        let mut key2 = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut key1);
        rand::thread_rng().fill_bytes(&mut key2);
        
        let plaintext = b"Test data";
        let encrypted = encrypt_data(plaintext, &key1).unwrap();
        
        assert!(decrypt_data(&encrypted, &key2).is_err());
    }
}use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use anyhow::{Context, Result};
use rand::RngCore;

const NONCE_SIZE: usize = 12;

pub fn encrypt_data(plaintext: &[u8], key: &[u8]) -> Result<Vec<u8>> {
    let cipher = Aes256Gcm::new_from_slice(key).context("Invalid key length")?;
    let mut nonce_bytes = [0u8; NONCE_SIZE];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    
    let mut ciphertext = cipher
        .encrypt(nonce, plaintext)
        .context("Encryption failed")?;
    
    let mut result = Vec::with_capacity(NONCE_SIZE + ciphertext.len());
    result.extend_from_slice(nonce_bytes);
    result.append(&mut ciphertext);
    Ok(result)
}

pub fn decrypt_data(ciphertext: &[u8], key: &[u8]) -> Result<Vec<u8>> {
    if ciphertext.len() < NONCE_SIZE {
        anyhow::bail!("Ciphertext too short");
    }
    
    let cipher = Aes256Gcm::new_from_slice(key).context("Invalid key length")?;
    let (nonce_bytes, encrypted_data) = ciphertext.split_at(NONCE_SIZE);
    let nonce = Nonce::from_slice(nonce_bytes);
    
    let plaintext = cipher
        .decrypt(nonce, encrypted_data)
        .context("Decryption failed")?;
    
    Ok(plaintext)
}

#[cfg(test)]
mod tests {
    use super::*;
    use hex::encode;

    #[test]
    fn test_encryption_roundtrip() {
        let key = [0x42u8; 32];
        let plaintext = b"Secret message for encryption test";
        
        let ciphertext = encrypt_data(plaintext, &key).unwrap();
        let decrypted = decrypt_data(&ciphertext, &key).unwrap();
        
        assert_eq!(plaintext.to_vec(), decrypted);
        assert_ne!(plaintext, &ciphertext[..]);
    }
    
    #[test]
    fn test_different_keys_fail() {
        let key1 = [0x41u8; 32];
        let key2 = [0x42u8; 32];
        let plaintext = b"Test data";
        
        let ciphertext = encrypt_data(plaintext, &key1).unwrap();
        let result = decrypt_data(&ciphertext, &key2);
        
        assert!(result.is_err());
    }
}