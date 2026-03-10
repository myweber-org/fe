use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use anyhow::{Context, Result};

const NONCE_SIZE: usize = 12;

pub fn encrypt_aes256_gcm(plaintext: &[u8], key: &[u8]) -> Result<Vec<u8>> {
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let nonce = Nonce::from_slice(&OsRng.gen::<[u8; NONCE_SIZE]>());

    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .context("Encryption failed")?;

    let mut result = Vec::with_capacity(NONCE_SIZE + ciphertext.len());
    result.extend_from_slice(nonce.as_slice());
    result.extend_from_slice(&ciphertext);

    Ok(result)
}

pub fn decrypt_aes256_gcm(ciphertext: &[u8], key: &[u8]) -> Result<Vec<u8>> {
    if ciphertext.len() < NONCE_SIZE {
        anyhow::bail!("Ciphertext too short");
    }

    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let (nonce_bytes, encrypted_data) = ciphertext.split_at(NONCE_SIZE);
    let nonce = Nonce::from_slice(nonce_bytes);

    cipher
        .decrypt(nonce, encrypted_data)
        .context("Decryption failed")
}use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use anyhow::{anyhow, Result};

const NONCE_SIZE: usize = 12;

pub fn encrypt_aes256gcm(plaintext: &[u8], key: &[u8]) -> Result<Vec<u8>> {
    if key.len() != 32 {
        return Err(anyhow!("Key must be 32 bytes for AES-256"));
    }
    
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let nonce = Nonce::from_slice(&generate_nonce());
    
    cipher.encrypt(nonce, plaintext)
        .map_err(|e| anyhow!("Encryption failed: {}", e))
}

pub fn decrypt_aes256gcm(ciphertext: &[u8], key: &[u8]) -> Result<Vec<u8>> {
    if key.len() != 32 {
        return Err(anyhow!("Key must be 32 bytes for AES-256"));
    }
    
    if ciphertext.len() <= NONCE_SIZE {
        return Err(anyhow!("Ciphertext too short"));
    }
    
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let (nonce_bytes, encrypted_data) = ciphertext.split_at(NONCE_SIZE);
    let nonce = Nonce::from_slice(nonce_bytes);
    
    cipher.decrypt(nonce, encrypted_data)
        .map_err(|e| anyhow!("Decryption failed: {}", e))
}

fn generate_nonce() -> [u8; NONCE_SIZE] {
    let mut nonce = [0u8; NONCE_SIZE];
    OsRng.fill_bytes(&mut nonce);
    nonce
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::RngCore;

    #[test]
    fn test_encryption_roundtrip() {
        let mut key = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut key);
        
        let plaintext = b"Secret message for encryption test";
        
        let ciphertext = encrypt_aes256gcm(plaintext, &key).unwrap();
        let decrypted = decrypt_aes256gcm(&ciphertext, &key).unwrap();
        
        assert_eq!(plaintext.to_vec(), decrypted);
    }

    #[test]
    fn test_wrong_key_fails() {
        let mut key1 = [0u8; 32];
        let mut key2 = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut key1);
        rand::thread_rng().fill_bytes(&mut key2);
        
        let plaintext = b"Test data";
        let ciphertext = encrypt_aes256gcm(plaintext, &key1).unwrap();
        
        assert!(decrypt_aes256gcm(&ciphertext, &key2).is_err());
    }
}