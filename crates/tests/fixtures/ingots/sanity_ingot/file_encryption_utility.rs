use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use argon2::{Argon2, PasswordHasher};
use rand::RngCore;
use std::fs;
use std::io::{Read, Write};

const NONCE_SIZE: usize = 12;
const SALT_SIZE: usize = 16;

pub struct FileEncryptor {
    key: [u8; 32],
}

impl FileEncryptor {
    pub fn new(password: &str, salt: &[u8; SALT_SIZE]) -> Self {
        let argon2 = Argon2::default();
        let mut key = [0u8; 32];
        argon2
            .hash_password_into(password.as_bytes(), salt, &mut key)
            .expect("Key derivation failed");
        Self { key }
    }

    pub fn encrypt_file(&self, input_path: &str, output_path: &str) -> Result<(), String> {
        let mut file_content = Vec::new();
        fs::File::open(input_path)
            .map_err(|e| format!("Failed to open input file: {}", e))?
            .read_to_end(&mut file_content)
            .map_err(|e| format!("Failed to read file: {}", e))?;

        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&self.key));
        let mut nonce = [0u8; NONCE_SIZE];
        OsRng.fill_bytes(&mut nonce);

        let encrypted_data = cipher
            .encrypt(Nonce::from_slice(&nonce), file_content.as_ref())
            .map_err(|e| format!("Encryption failed: {}", e))?;

        let mut output = fs::File::create(output_path)
            .map_err(|e| format!("Failed to create output file: {}", e))?;

        output
            .write_all(&nonce)
            .map_err(|e| format!("Failed to write nonce: {}", e))?;
        output
            .write_all(&encrypted_data)
            .map_err(|e| format!("Failed to write encrypted data: {}", e))?;

        Ok(())
    }

    pub fn decrypt_file(&self, input_path: &str, output_path: &str) -> Result<(), String> {
        let mut encrypted_content = Vec::new();
        fs::File::open(input_path)
            .map_err(|e| format!("Failed to open encrypted file: {}", e))?
            .read_to_end(&mut encrypted_content)
            .map_err(|e| format!("Failed to read encrypted file: {}", e))?;

        if encrypted_content.len() < NONCE_SIZE {
            return Err("File too short to contain nonce".to_string());
        }

        let (nonce_bytes, ciphertext) = encrypted_content.split_at(NONCE_SIZE);
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&self.key));

        let decrypted_data = cipher
            .decrypt(Nonce::from_slice(nonce_bytes), ciphertext)
            .map_err(|e| format!("Decryption failed: {}", e))?;

        fs::write(output_path, decrypted_data)
            .map_err(|e| format!("Failed to write decrypted file: {}", e))?;

        Ok(())
    }
}

pub fn generate_salt() -> [u8; SALT_SIZE] {
    let mut salt = [0u8; SALT_SIZE];
    OsRng.fill_bytes(&mut salt);
    salt
}
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use sha2::{Sha256, Digest};
use std::fs;
use std::io::{Read, Write};
use std::path::Path;

const NONCE_SIZE: usize = 12;
const KEY_SIZE: usize = 32;
const HMAC_SIZE: usize = 32;

pub struct FileEncryptor {
    cipher: Aes256Gcm,
    hmac_key: [u8; KEY_SIZE],
}

impl FileEncryptor {
    pub fn new() -> Self {
        let key = Key::<Aes256Gcm>::generate(&mut OsRng);
        let mut hmac_key = [0u8; KEY_SIZE];
        OsRng.fill_bytes(&mut hmac_key);
        
        FileEncryptor {
            cipher: Aes256Gcm::new(&key),
            hmac_key,
        }
    }

    pub fn from_key(key_bytes: &[u8], hmac_key_bytes: &[u8]) -> Result<Self, &'static str> {
        if key_bytes.len() != KEY_SIZE || hmac_key_bytes.len() != KEY_SIZE {
            return Err("Invalid key length");
        }
        
        let key = Key::<Aes256Gcm>::from_slice(key_bytes);
        let mut hmac_key = [0u8; KEY_SIZE];
        hmac_key.copy_from_slice(hmac_key_bytes);
        
        Ok(FileEncryptor {
            cipher: Aes256Gcm::new(key),
            hmac_key,
        })
    }

    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), String> {
        let mut file_data = Vec::new();
        fs::File::open(input_path)
            .map_err(|e| format!("Failed to open input file: {}", e))?
            .read_to_end(&mut file_data)
            .map_err(|e| format!("Failed to read input file: {}", e))?;

        let nonce = Nonce::generate(&mut OsRng);
        let ciphertext = self.cipher
            .encrypt(&nonce, file_data.as_ref())
            .map_err(|e| format!("Encryption failed: {}", e))?;

        let hmac = self.calculate_hmac(&ciphertext);
        
        let mut output_data = Vec::with_capacity(NONCE_SIZE + ciphertext.len() + HMAC_SIZE);
        output_data.extend_from_slice(nonce.as_slice());
        output_data.extend_from_slice(&ciphertext);
        output_data.extend_from_slice(&hmac);

        fs::File::create(output_path)
            .map_err(|e| format!("Failed to create output file: {}", e))?
            .write_all(&output_data)
            .map_err(|e| format!("Failed to write output file: {}", e))?;

        Ok(())
    }

    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), String> {
        let mut encrypted_data = Vec::new();
        fs::File::open(input_path)
            .map_err(|e| format!("Failed to open input file: {}", e))?
            .read_to_end(&mut encrypted_data)
            .map_err(|e| format!("Failed to read input file: {}", e))?;

        if encrypted_data.len() < NONCE_SIZE + HMAC_SIZE {
            return Err("Invalid encrypted file format".to_string());
        }

        let nonce = Nonce::from_slice(&encrypted_data[..NONCE_SIZE]);
        let ciphertext_end = encrypted_data.len() - HMAC_SIZE;
        let ciphertext = &encrypted_data[NONCE_SIZE..ciphertext_end];
        let received_hmac = &encrypted_data[ciphertext_end..];

        let calculated_hmac = self.calculate_hmac(ciphertext);
        if received_hmac != calculated_hmac {
            return Err("HMAC verification failed - file may be tampered".to_string());
        }

        let plaintext = self.cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| format!("Decryption failed: {}", e))?;

        fs::File::create(output_path)
            .map_err(|e| format!("Failed to create output file: {}", e))?
            .write_all(&plaintext)
            .map_err(|e| format!("Failed to write output file: {}", e))?;

        Ok(())
    }

    fn calculate_hmac(&self, data: &[u8]) -> [u8; HMAC_SIZE] {
        let mut hasher = Sha256::new();
        hasher.update(&self.hmac_key);
        hasher.update(data);
        let result = hasher.finalize();
        let mut hmac = [0u8; HMAC_SIZE];
        hmac.copy_from_slice(&result);
        hmac
    }

    pub fn export_keys(&self) -> (Vec<u8>, Vec<u8>) {
        let cipher_key = self.cipher.key().as_slice().to_vec();
        let hmac_key = self.hmac_key.to_vec();
        (cipher_key, hmac_key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let encryptor = FileEncryptor::new();
        let test_data = b"Test encryption data for AES-256-GCM with HMAC verification";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), test_data).unwrap();
        
        encryptor.encrypt_file(input_file.path(), encrypted_file.path()).unwrap();
        encryptor.decrypt_file(encrypted_file.path(), decrypted_file.path()).unwrap();
        
        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(test_data.to_vec(), decrypted_data);
    }

    #[test]
    fn test_key_export_import() {
        let original = FileEncryptor::new();
        let (cipher_key, hmac_key) = original.export_keys();
        
        let restored = FileEncryptor::from_key(&cipher_key, &hmac_key).unwrap();
        
        let test_data = b"Test key export/import functionality";
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), test_data).unwrap();
        
        original.encrypt_file(input_file.path(), encrypted_file.path()).unwrap();
        restored.decrypt_file(encrypted_file.path(), decrypted_file.path()).unwrap();
        
        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(test_data.to_vec(), decrypted_data);
    }
}