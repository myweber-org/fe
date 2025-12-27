use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use std::error::Error;

pub fn encrypt_data(plaintext: &[u8], key: &[u8; 32]) -> Result<Vec<u8>, Box<dyn Error>> {
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let nonce = Nonce::from_slice(&[0u8; 12]);
    let ciphertext = cipher.encrypt(nonce, plaintext)?;
    Ok(ciphertext)
}

pub fn decrypt_data(ciphertext: &[u8], key: &[u8; 32]) -> Result<Vec<u8>, Box<dyn Error>> {
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let nonce = Nonce::from_slice(&[0u8; 12]);
    let plaintext = cipher.decrypt(nonce, ciphertext)?;
    Ok(plaintext)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_decryption() {
        let key = [0u8; 32];
        let plaintext = b"secret message";
        
        let ciphertext = encrypt_data(plaintext, &key).unwrap();
        let decrypted = decrypt_data(&ciphertext, &key).unwrap();
        
        assert_eq!(plaintext.to_vec(), decrypted);
    }
}