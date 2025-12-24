use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use std::error::Error;

pub fn encrypt_data(plaintext: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
    let key = Aes256Gcm::generate_key(&mut OsRng);
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(b"unique_nonce_");
    
    let ciphertext = cipher.encrypt(nonce, plaintext)
        .map_err(|e| format!("Encryption failed: {}", e))?;
    
    let mut result = Vec::new();
    result.extend_from_slice(&key);
    result.extend_from_slice(nonce);
    result.extend_from_slice(&ciphertext);
    
    Ok(result)
}

pub fn decrypt_data(ciphertext: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
    if ciphertext.len() < 48 {
        return Err("Invalid ciphertext length".into());
    }
    
    let key = Key::<Aes256Gcm>::from_slice(&ciphertext[0..32]);
    let nonce = Nonce::from_slice(&ciphertext[32..44]);
    let encrypted_data = &ciphertext[44..];
    
    let cipher = Aes256Gcm::new(key);
    let plaintext = cipher.decrypt(nonce, encrypted_data)
        .map_err(|e| format!("Decryption failed: {}", e))?;
    
    Ok(plaintext)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_roundtrip() {
        let original_data = b"Secret message for encryption test";
        
        let encrypted = encrypt_data(original_data).unwrap();
        let decrypted = decrypt_data(&encrypted).unwrap();
        
        assert_eq!(original_data.to_vec(), decrypted);
    }
}