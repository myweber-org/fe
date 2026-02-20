
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use chacha20poly1305::{ChaCha20Poly1305, Key as ChaChaKey};
use std::error::Error;

#[derive(Debug)]
pub enum EncryptionAlgorithm {
    Aes256Gcm,
    ChaCha20Poly1305,
}

pub struct EncryptionResult {
    pub ciphertext: Vec<u8>,
    pub nonce: Vec<u8>,
}

pub fn encrypt_data(
    plaintext: &[u8],
    algorithm: EncryptionAlgorithm,
) -> Result<EncryptionResult, Box<dyn Error>> {
    match algorithm {
        EncryptionAlgorithm::Aes256Gcm => {
            let key = Aes256Gcm::generate_key(&mut OsRng);
            let cipher = Aes256Gcm::new(&key);
            let nonce = Nonce::generate(&mut OsRng);
            
            let ciphertext = cipher
                .encrypt(&nonce, plaintext)
                .map_err(|e| format!("AES encryption failed: {}", e))?;
            
            Ok(EncryptionResult {
                ciphertext,
                nonce: nonce.to_vec(),
            })
        }
        EncryptionAlgorithm::ChaCha20Poly1305 => {
            let key = ChaCha20Poly1305::generate_key(&mut OsRng);
            let cipher = ChaCha20Poly1305::new(&key);
            let nonce = chacha20poly1305::Nonce::generate(&mut OsRng);
            
            let ciphertext = cipher
                .encrypt(&nonce, plaintext)
                .map_err(|e| format!("ChaCha20 encryption failed: {}", e))?;
            
            Ok(EncryptionResult {
                ciphertext,
                nonce: nonce.to_vec(),
            })
        }
    }
}

pub fn decrypt_data(
    ciphertext: &[u8],
    nonce: &[u8],
    algorithm: EncryptionAlgorithm,
) -> Result<Vec<u8>, Box<dyn Error>> {
    match algorithm {
        EncryptionAlgorithm::Aes256Gcm => {
            let key = Aes256Gcm::generate_key(&mut OsRng);
            let cipher = Aes256Gcm::new(&key);
            let nonce = Nonce::from_slice(nonce);
            
            cipher
                .decrypt(nonce, ciphertext)
                .map_err(|e| format!("AES decryption failed: {}", e).into())
        }
        EncryptionAlgorithm::ChaCha20Poly1305 => {
            let key = ChaCha20Poly1305::generate_key(&mut OsRng);
            let cipher = ChaCha20Poly1305::new(&key);
            let nonce = chacha20poly1305::Nonce::from_slice(nonce);
            
            cipher
                .decrypt(nonce, ciphertext)
                .map_err(|e| format!("ChaCha20 decryption failed: {}", e).into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aes_encryption_decryption() {
        let plaintext = b"Test secret message";
        let result = encrypt_data(plaintext, EncryptionAlgorithm::Aes256Gcm).unwrap();
        
        let decrypted = decrypt_data(
            &result.ciphertext,
            &result.nonce,
            EncryptionAlgorithm::Aes256Gcm,
        ).unwrap();
        
        assert_eq!(plaintext.to_vec(), decrypted);
    }

    #[test]
    fn test_chacha_encryption_decryption() {
        let plaintext = b"Another confidential data";
        let result = encrypt_data(plaintext, EncryptionAlgorithm::ChaCha20Poly1305).unwrap();
        
        let decrypted = decrypt_data(
            &result.ciphertext,
            &result.nonce,
            EncryptionAlgorithm::ChaCha20Poly1305,
        ).unwrap();
        
        assert_eq!(plaintext.to_vec(), decrypted);
    }
}