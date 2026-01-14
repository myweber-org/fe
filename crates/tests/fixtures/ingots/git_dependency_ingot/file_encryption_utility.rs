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
}

pub struct FileEncryptor {
    algorithm: CipherAlgorithm,
}

impl FileEncryptor {
    pub fn new(algorithm: CipherAlgorithm) -> Self {
        Self { algorithm }
    }

    pub fn encrypt(&self, plaintext: &[u8], key: &[u8]) -> Result<EncryptionResult, String> {
        match self.algorithm {
            CipherAlgorithm::Aes256Gcm => self.encrypt_aes(plaintext, key),
            CipherAlgorithm::ChaCha20Poly1305 => self.encrypt_chacha(plaintext, key),
        }
    }

    fn encrypt_aes(&self, plaintext: &[u8], key: &[u8]) -> Result<EncryptionResult, String> {
        if key.len() != 32 {
            return Err("AES-256-GCM requires 32-byte key".to_string());
        }

        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        cipher
            .encrypt(nonce, plaintext)
            .map(|ciphertext| EncryptionResult {
                ciphertext,
                nonce: nonce_bytes.to_vec(),
            })
            .map_err(|e| format!("Encryption failed: {}", e))
    }

    fn encrypt_chacha(&self, plaintext: &[u8], key: &[u8]) -> Result<EncryptionResult, String> {
        if key.len() != 32 {
            return Err("ChaCha20Poly1305 requires 32-byte key".to_string());
        }

        let cipher = ChaCha20Poly1305::new(ChaChaKey::from_slice(key));
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = ChaChaNonce::from_slice(&nonce_bytes);

        cipher
            .encrypt(nonce, plaintext)
            .map(|ciphertext| EncryptionResult {
                ciphertext,
                nonce: nonce_bytes.to_vec(),
            })
            .map_err(|e| format!("Encryption failed: {}", e))
    }

    pub fn decrypt(&self, ciphertext: &[u8], key: &[u8], nonce: &[u8]) -> Result<Vec<u8>, String> {
        match self.algorithm {
            CipherAlgorithm::Aes256Gcm => self.decrypt_aes(ciphertext, key, nonce),
            CipherAlgorithm::ChaCha20Poly1305 => self.decrypt_chacha(ciphertext, key, nonce),
        }
    }

    fn decrypt_aes(&self, ciphertext: &[u8], key: &[u8], nonce: &[u8]) -> Result<Vec<u8>, String> {
        if key.len() != 32 {
            return Err("AES-256-GCM requires 32-byte key".to_string());
        }
        if nonce.len() != 12 {
            return Err("AES-256-GCM requires 12-byte nonce".to_string());
        }

        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
        let nonce = Nonce::from_slice(nonce);

        cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| format!("Decryption failed: {}", e))
    }

    fn decrypt_chacha(&self, ciphertext: &[u8], key: &[u8], nonce: &[u8]) -> Result<Vec<u8>, String> {
        if key.len() != 32 {
            return Err("ChaCha20Poly1305 requires 32-byte key".to_string());
        }
        if nonce.len() != 12 {
            return Err("ChaCha20Poly1305 requires 12-byte nonce".to_string());
        }

        let cipher = ChaCha20Poly1305::new(ChaChaKey::from_slice(key));
        let nonce = ChaChaNonce::from_slice(nonce);

        cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| format!("Decryption failed: {}", e))
    }
}

pub fn generate_random_key() -> Vec<u8> {
    let mut key = vec![0u8; 32];
    OsRng.fill_bytes(&mut key);
    key
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aes_encryption_decryption() {
        let encryptor = FileEncryptor::new(CipherAlgorithm::Aes256Gcm);
        let key = generate_random_key();
        let plaintext = b"Test secret message";
        
        let result = encryptor.encrypt(plaintext, &key).unwrap();
        let decrypted = encryptor.decrypt(&result.ciphertext, &key, &result.nonce).unwrap();
        
        assert_eq!(plaintext.to_vec(), decrypted);
    }

    #[test]
    fn test_chacha_encryption_decryption() {
        let encryptor = FileEncryptor::new(CipherAlgorithm::ChaCha20Poly1305);
        let key = generate_random_key();
        let plaintext = b"Another secret message";
        
        let result = encryptor.encrypt(plaintext, &key).unwrap();
        let decrypted = encryptor.decrypt(&result.ciphertext, &key, &result.nonce).unwrap();
        
        assert_eq!(plaintext.to_vec(), decrypted);
    }

    #[test]
    fn test_wrong_key_fails() {
        let encryptor = FileEncryptor::new(CipherAlgorithm::Aes256Gcm);
        let key = generate_random_key();
        let wrong_key = generate_random_key();
        let plaintext = b"Secret data";
        
        let result = encryptor.encrypt(plaintext, &key).unwrap();
        let decryption_result = encryptor.decrypt(&result.ciphertext, &wrong_key, &result.nonce);
        
        assert!(decryption_result.is_err());
    }
}