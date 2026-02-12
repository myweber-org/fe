
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use chacha20poly1305::{ChaCha20Poly1305, Key as ChaChaKey, Nonce as ChaChaNonce};
use rand::RngCore;

pub enum CipherAlgorithm {
    Aes256Gcm,
    ChaCha20Poly1305,
}

pub struct EncryptionResult {
    pub ciphertext: Vec<u8>,
    pub nonce: Vec<u8>,
    pub algorithm: CipherAlgorithm,
}

pub fn encrypt_data(
    plaintext: &[u8],
    algorithm: CipherAlgorithm,
    key: Option<&[u8]>,
) -> Result<EncryptionResult, Box<dyn std::error::Error>> {
    let mut rng = OsRng;
    
    match algorithm {
        CipherAlgorithm::Aes256Gcm => {
            let key = if let Some(k) = key {
                if k.len() != 32 {
                    return Err("AES-256-GCM requires 32-byte key".into());
                }
                Key::<Aes256Gcm>::from_slice(k)
            } else {
                let mut random_key = [0u8; 32];
                rng.fill_bytes(&mut random_key);
                Key::<Aes256Gcm>::from_slice(&random_key)
            };
            
            let cipher = Aes256Gcm::new(key);
            let mut nonce_bytes = [0u8; 12];
            rng.fill_bytes(&mut nonce_bytes);
            let nonce = Nonce::from_slice(&nonce_bytes);
            
            let ciphertext = cipher
                .encrypt(nonce, plaintext)
                .map_err(|e| format!("Encryption failed: {}", e))?;
            
            Ok(EncryptionResult {
                ciphertext,
                nonce: nonce_bytes.to_vec(),
                algorithm: CipherAlgorithm::Aes256Gcm,
            })
        }
        
        CipherAlgorithm::ChaCha20Poly1305 => {
            let key = if let Some(k) = key {
                if k.len() != 32 {
                    return Err("ChaCha20Poly1305 requires 32-byte key".into());
                }
                ChaChaKey::from_slice(k)
            } else {
                let mut random_key = [0u8; 32];
                rng.fill_bytes(&mut random_key);
                ChaChaKey::from_slice(&random_key)
            };
            
            let cipher = ChaCha20Poly1305::new(key);
            let mut nonce_bytes = [0u8; 12];
            rng.fill_bytes(&mut nonce_bytes);
            let nonce = ChaChaNonce::from_slice(&nonce_bytes);
            
            let ciphertext = cipher
                .encrypt(nonce, plaintext)
                .map_err(|e| format!("Encryption failed: {}", e))?;
            
            Ok(EncryptionResult {
                ciphertext,
                nonce: nonce_bytes.to_vec(),
                algorithm: CipherAlgorithm::ChaCha20Poly1305,
            })
        }
    }
}

pub fn decrypt_data(
    encrypted: &EncryptionResult,
    key: &[u8],
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    match encrypted.algorithm {
        CipherAlgorithm::Aes256Gcm => {
            if key.len() != 32 {
                return Err("AES-256-GCM requires 32-byte key".into());
            }
            
            let cipher_key = Key::<Aes256Gcm>::from_slice(key);
            let cipher = Aes256Gcm::new(cipher_key);
            let nonce = Nonce::from_slice(&encrypted.nonce);
            
            cipher
                .decrypt(nonce, encrypted.ciphertext.as_ref())
                .map_err(|e| format!("Decryption failed: {}", e).into())
        }
        
        CipherAlgorithm::ChaCha20Poly1305 => {
            if key.len() != 32 {
                return Err("ChaCha20Poly1305 requires 32-byte key".into());
            }
            
            let cipher_key = ChaChaKey::from_slice(key);
            let cipher = ChaCha20Poly1305::new(cipher_key);
            let nonce = ChaChaNonce::from_slice(&encrypted.nonce);
            
            cipher
                .decrypt(nonce, encrypted.ciphertext.as_ref())
                .map_err(|e| format!("Decryption failed: {}", e).into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_aes_encryption_decryption() {
        let plaintext = b"Secret message for AES";
        let key = b"32-byte-key-for-aes-256-gcm-test!!";
        
        let encrypted = encrypt_data(
            plaintext,
            CipherAlgorithm::Aes256Gcm,
            Some(key),
        ).unwrap();
        
        let decrypted = decrypt_data(&encrypted, key).unwrap();
        
        assert_eq!(plaintext.to_vec(), decrypted);
    }
    
    #[test]
    fn test_chacha_encryption_decryption() {
        let plaintext = b"Secret message for ChaCha";
        let key = b"32-byte-key-for-chacha-test-123!!";
        
        let encrypted = encrypt_data(
            plaintext,
            CipherAlgorithm::ChaCha20Poly1305,
            Some(key),
        ).unwrap();
        
        let decrypted = decrypt_data(&encrypted, key).unwrap();
        
        assert_eq!(plaintext.to_vec(), decrypted);
    }
    
    #[test]
    fn test_key_generation() {
        let plaintext = b"Message with generated key";
        
        let encrypted = encrypt_data(
            plaintext,
            CipherAlgorithm::Aes256Gcm,
            None,
        ).unwrap();
        
        assert_eq!(encrypted.nonce.len(), 12);
        assert!(!encrypted.ciphertext.is_empty());
    }
}
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use std::fs;

pub fn encrypt_file(input_path: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let key = Aes256Gcm::generate_key(&mut OsRng);
    let cipher = Aes256Gcm::new(&key);
    
    let data = fs::read(input_path)?;
    let nonce = Nonce::from_slice(b"unique_nonce_");
    
    let encrypted_data = cipher.encrypt(nonce, data.as_ref())
        .map_err(|e| format!("Encryption failed: {}", e))?;
    
    fs::write(output_path, encrypted_data)?;
    
    let key_path = format!("{}.key", output_path);
    fs::write(key_path, key.as_slice())?;
    
    Ok(())
}

pub fn decrypt_file(input_path: &str, key_path: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let key_data = fs::read(key_path)?;
    let key = key_data.as_slice().try_into()
        .map_err(|_| "Invalid key length")?;
    
    let cipher = Aes256Gcm::new(&key);
    let encrypted_data = fs::read(input_path)?;
    let nonce = Nonce::from_slice(b"unique_nonce_");
    
    let decrypted_data = cipher.decrypt(nonce, encrypted_data.as_ref())
        .map_err(|e| format!("Decryption failed: {}", e))?;
    
    fs::write(output_path, decrypted_data)?;
    Ok(())
}