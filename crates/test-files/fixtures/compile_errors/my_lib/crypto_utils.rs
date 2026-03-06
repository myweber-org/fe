use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use std::error::Error;

pub fn encrypt_data(plaintext: &[u8], key: &[u8; 32]) -> Result<Vec<u8>, Box<dyn Error>> {
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let nonce = Nonce::generate(&mut OsRng);
    let ciphertext = cipher.encrypt(&nonce, plaintext)?;
    let mut result = Vec::with_capacity(nonce.len() + ciphertext.len());
    result.extend_from_slice(nonce.as_slice());
    result.extend_from_slice(&ciphertext);
    Ok(result)
}

pub fn decrypt_data(ciphertext_with_nonce: &[u8], key: &[u8; 32]) -> Result<Vec<u8>, Box<dyn Error>> {
    if ciphertext_with_nonce.len() < 12 {
        return Err("Invalid ciphertext length".into());
    }
    let (nonce_slice, ciphertext) = ciphertext_with_nonce.split_at(12);
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let nonce = Nonce::from_slice(nonce_slice);
    let plaintext = cipher.decrypt(nonce, ciphertext)?;
    Ok(plaintext)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_roundtrip() {
        let key = [0x42; 32];
        let plaintext = b"Secret message for encryption test";
        let encrypted = encrypt_data(plaintext, &key).unwrap();
        let decrypted = decrypt_data(&encrypted, &key).unwrap();
        assert_eq!(plaintext.to_vec(), decrypted);
    }

    #[test]
    fn test_decryption_fails_with_wrong_key() {
        let key = [0x42; 32];
        let wrong_key = [0x99; 32];
        let plaintext = b"Another secret";
        let encrypted = encrypt_data(plaintext, &key).unwrap();
        assert!(decrypt_data(&encrypted, &wrong_key).is_err());
    }
}