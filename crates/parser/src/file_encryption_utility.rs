use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use pbkdf2::{pbkdf2_hmac, Params};
use rand::RngCore;
use sha2::Sha256;

type Aes256CbcEnc = cbc::Encryptor<aes::Aes256>;
type Aes256CbcDec = cbc::Decryptor<aes::Aes256>;

const SALT_LENGTH: usize = 16;
const IV_LENGTH: usize = 16;
const KEY_LENGTH: usize = 32;
const PBKDF2_ITERATIONS: u32 = 100_000;

pub struct EncryptionResult {
    pub ciphertext: Vec<u8>,
    pub salt: [u8; SALT_LENGTH],
    pub iv: [u8; IV_LENGTH],
}

pub fn encrypt_data(data: &[u8], password: &str) -> Result<EncryptionResult, String> {
    let mut salt = [0u8; SALT_LENGTH];
    let mut iv = [0u8; IV_LENGTH];
    
    rand::thread_rng().fill_bytes(&mut salt);
    rand::thread_rng().fill_bytes(&mut iv);
    
    let key = derive_key(password, &salt)?;
    
    let ciphertext = Aes256CbcEnc::new(&key.into(), &iv.into())
        .encrypt_padded_vec_mut::<Pkcs7>(data);
    
    Ok(EncryptionResult {
        ciphertext,
        salt,
        iv,
    })
}

pub fn decrypt_data(
    encrypted: &EncryptionResult, 
    password: &str
) -> Result<Vec<u8>, String> {
    let key = derive_key(password, &encrypted.salt)?;
    
    let plaintext = Aes256CbcDec::new(&key.into(), &encrypted.iv.into())
        .decrypt_padded_vec_mut::<Pkcs7>(&encrypted.ciphertext)
        .map_err(|e| format!("Decryption failed: {}", e))?;
    
    Ok(plaintext)
}

fn derive_key(password: &str, salt: &[u8]) -> Result<[u8; KEY_LENGTH], String> {
    let mut key = [0u8; KEY_LENGTH];
    
    let params = Params {
        rounds: PBKDF2_ITERATIONS,
        output_length: KEY_LENGTH,
    };
    
    pbkdf2_hmac::<Sha256>(password.as_bytes(), salt, params.rounds, &mut key)
        .map_err(|e| format!("Key derivation failed: {}", e))?;
    
    Ok(key)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_encryption_decryption() {
        let original_data = b"Secret message for encryption test";
        let password = "StrongPassword123!";
        
        let encrypted = encrypt_data(original_data, password).unwrap();
        let decrypted = decrypt_data(&encrypted, password).unwrap();
        
        assert_eq!(original_data.to_vec(), decrypted);
    }
    
    #[test]
    fn test_wrong_password_fails() {
        let original_data = b"Test data";
        let password = "CorrectPassword";
        let wrong_password = "WrongPassword";
        
        let encrypted = encrypt_data(original_data, password).unwrap();
        let result = decrypt_data(&encrypted, wrong_password);
        
        assert!(result.is_err());
    }
}use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;

pub struct XorCipher {
    key: Vec<u8>,
}

impl XorCipher {
    pub fn new(key: &str) -> Self {
        XorCipher {
            key: key.as_bytes().to_vec(),
        }
    }

    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), String> {
        self.process_file(input_path, output_path, true)
    }

    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), String> {
        self.process_file(input_path, output_path, false)
    }

    fn process_file(&self, input_path: &Path, output_path: &Path, _is_encrypt: bool) -> Result<(), String> {
        if self.key.is_empty() {
            return Err("Encryption key cannot be empty".to_string());
        }

        let mut input_file = File::open(input_path)
            .map_err(|e| format!("Failed to open input file: {}", e))?;

        let mut buffer = Vec::new();
        input_file.read_to_end(&mut buffer)
            .map_err(|e| format!("Failed to read input file: {}", e))?;

        let processed_data: Vec<u8> = buffer
            .iter()
            .enumerate()
            .map(|(i, &byte)| byte ^ self.key[i % self.key.len()])
            .collect();

        let mut output_file = File::create(output_path)
            .map_err(|e| format!("Failed to create output file: {}", e))?;

        output_file.write_all(&processed_data)
            .map_err(|e| format!("Failed to write output file: {}", e))?;

        Ok(())
    }

    pub fn encrypt_string(&self, text: &str) -> Vec<u8> {
        text.as_bytes()
            .iter()
            .enumerate()
            .map(|(i, &byte)| byte ^ self.key[i % self.key.len()])
            .collect()
    }

    pub fn decrypt_string(&self, data: &[u8]) -> String {
        let decrypted: Vec<u8> = data
            .iter()
            .enumerate()
            .map(|(i, &byte)| byte ^ self.key[i % self.key.len()])
            .collect();

        String::from_utf8_lossy(&decrypted).to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_string_encryption_decryption() {
        let cipher = XorCipher::new("secret_key");
        let original_text = "Hello, World!";
        
        let encrypted = cipher.encrypt_string(original_text);
        let decrypted = cipher.decrypt_string(&encrypted);
        
        assert_eq!(original_text, decrypted);
    }

    #[test]
    fn test_file_encryption_decryption() {
        let cipher = XorCipher::new("test_key_123");
        let original_content = b"Sample file content for encryption testing.";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), original_content).unwrap();
        
        cipher.encrypt_file(input_file.path(), encrypted_file.path()).unwrap();
        cipher.decrypt_file(encrypted_file.path(), decrypted_file.path()).unwrap();
        
        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(original_content.to_vec(), decrypted_content);
    }

    #[test]
    fn test_empty_key() {
        let cipher = XorCipher::new("");
        let temp_file = NamedTempFile::new().unwrap();
        
        let result = cipher.encrypt_file(temp_file.path(), temp_file.path());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("cannot be empty"));
    }
}