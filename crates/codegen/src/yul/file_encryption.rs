
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: u8 = 0xAA;

pub fn encrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    let encryption_key = key.unwrap_or(DEFAULT_KEY);
    
    let mut input_file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    input_file.read_to_end(&mut buffer)?;
    
    let encrypted_data: Vec<u8> = buffer.iter()
        .map(|byte| byte ^ encryption_key)
        .collect();
    
    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&encrypted_data)?;
    
    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    encrypt_file(input_path, output_path, key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_encryption_decryption() {
        let original_text = b"Hello, World! This is a test message.";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), original_text).unwrap();
        
        encrypt_file(
            input_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap(),
            Some(0x55)
        ).unwrap();
        
        let encrypted_content = fs::read(encrypted_file.path()).unwrap();
        assert_ne!(encrypted_content, original_text);
        
        decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap(),
            Some(0x55)
        ).unwrap();
        
        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(decrypted_content, original_text);
    }
    
    #[test]
    fn test_default_key() {
        let test_data = b"Test data with default key";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), test_data).unwrap();
        
        encrypt_file(
            input_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap(),
            None
        ).unwrap();
        
        decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap(),
            None
        ).unwrap();
        
        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(decrypted_content, test_data);
    }
}
use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use hmac::{Hmac, Mac};
use pbkdf2::pbkdf2;
use rand::rngs::OsRng;
use rand::RngCore;
use sha2::Sha256;
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

type Aes256CbcEnc = cbc::Encryptor<aes::Aes256>;
type Aes256CbcDec = cbc::Decryptor<aes::Aes256>;

const SALT_LEN: usize = 16;
const IV_LEN: usize = 16;
const KEY_LEN: usize = 32;
const PBKDF2_ITERATIONS: u32 = 100_000;

#[derive(Debug)]
pub enum CryptoError {
    IoError(io::Error),
    InvalidData,
    KeyDerivationFailed,
}

impl From<io::Error> for CryptoError {
    fn from(err: io::Error) -> Self {
        CryptoError::IoError(err)
    }
}

pub struct FileCrypto;

impl FileCrypto {
    pub fn encrypt_file(
        input_path: &Path,
        output_path: &Path,
        password: &str,
    ) -> Result<(), CryptoError> {
        let mut input_file = fs::File::open(input_path)?;
        let mut plaintext = Vec::new();
        input_file.read_to_end(&mut plaintext)?;

        let mut salt = [0u8; SALT_LEN];
        let mut iv = [0u8; IV_LEN];
        OsRng.fill_bytes(&mut salt);
        OsRng.fill_bytes(&mut iv);

        let key = Self::derive_key(password, &salt)?;

        let cipher = Aes256CbcEnc::new(&key.into(), &iv.into());
        let ciphertext = cipher.encrypt_padded_vec_mut::<Pkcs7>(&plaintext);

        let mut output_file = fs::File::create(output_path)?;
        output_file.write_all(&salt)?;
        output_file.write_all(&iv)?;
        output_file.write_all(&ciphertext)?;

        Ok(())
    }

    pub fn decrypt_file(
        input_path: &Path,
        output_path: &Path,
        password: &str,
    ) -> Result<(), CryptoError> {
        let mut input_file = fs::File::open(input_path)?;
        let mut encrypted_data = Vec::new();
        input_file.read_to_end(&mut encrypted_data)?;

        if encrypted_data.len() < SALT_LEN + IV_LEN {
            return Err(CryptoError::InvalidData);
        }

        let salt = &encrypted_data[..SALT_LEN];
        let iv = &encrypted_data[SALT_LEN..SALT_LEN + IV_LEN];
        let ciphertext = &encrypted_data[SALT_LEN + IV_LEN..];

        let key = Self::derive_key(password, salt)?;

        let cipher = Aes256CbcDec::new(&key.into(), iv.into());
        let plaintext = cipher
            .decrypt_padded_vec_mut::<Pkcs7>(ciphertext)
            .map_err(|_| CryptoError::InvalidData)?;

        let mut output_file = fs::File::create(output_path)?;
        output_file.write_all(&plaintext)?;

        Ok(())
    }

    fn derive_key(password: &str, salt: &[u8]) -> Result<[u8; KEY_LEN], CryptoError> {
        let mut key = [0u8; KEY_LEN];
        pbkdf2::<Hmac<Sha256>>(password.as_bytes(), salt, PBKDF2_ITERATIONS, &mut key);
        Ok(key)
    }

    pub fn generate_hmac(data: &[u8], key: &[u8]) -> Result<Vec<u8>, CryptoError> {
        let mut mac = Hmac::<Sha256>::new_from_slice(key).map_err(|_| CryptoError::KeyDerivationFailed)?;
        mac.update(data);
        let result = mac.finalize().into_bytes().to_vec();
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encrypt_decrypt() {
        let plaintext = b"Secret data that needs protection";
        let password = "strong_password_123";

        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        fs::write(input_file.path(), plaintext).unwrap();

        FileCrypto::encrypt_file(input_file.path(), encrypted_file.path(), password).unwrap();
        FileCrypto::decrypt_file(encrypted_file.path(), decrypted_file.path(), password).unwrap();

        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(plaintext.to_vec(), decrypted_data);
    }

    #[test]
    fn test_hmac_generation() {
        let data = b"Data to authenticate";
        let key = b"secret_hmac_key";
        
        let hmac1 = FileCrypto::generate_hmac(data, key).unwrap();
        let hmac2 = FileCrypto::generate_hmac(data, key).unwrap();
        
        assert_eq!(hmac1, hmac2);
        assert_eq!(hmac1.len(), 32);
    }
}
use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use pbkdf2::{pbkdf2_hmac, Params};
use rand::RngCore;
use sha2::Sha256;
use std::fs;
use std::io::{Read, Write};

type Aes256CbcEnc = cbc::Encryptor<aes::Aes256>;
type Aes256CbcDec = cbc::Decryptor<aes::Aes256>;

const SALT_LENGTH: usize = 16;
const IV_LENGTH: usize = 16;
const KEY_ITERATIONS: u32 = 100_000;
const KEY_LENGTH: usize = 32;

pub struct EncryptionResult {
    pub ciphertext: Vec<u8>,
    pub salt: [u8; SALT_LENGTH],
    pub iv: [u8; IV_LENGTH],
}

pub fn derive_key(password: &str, salt: &[u8]) -> [u8; KEY_LENGTH] {
    let mut key = [0u8; KEY_LENGTH];
    let params = Params {
        rounds: KEY_ITERATIONS,
        output_length: KEY_LENGTH,
    };
    pbkdf2_hmac::<Sha256>(password.as_bytes(), salt, params.rounds, &mut key)
        .expect("PBKDF2 should not fail");
    key
}

pub fn encrypt_file(password: &str, input_path: &str, output_path: &str) -> Result<(), String> {
    let mut file_content = fs::read(input_path).map_err(|e| format!("Read error: {}", e))?;
    
    let mut salt = [0u8; SALT_LENGTH];
    let mut iv = [0u8; IV_LENGTH];
    rand::thread_rng().fill_bytes(&mut salt);
    rand::thread_rng().fill_bytes(&mut iv);
    
    let key = derive_key(password, &salt);
    
    let ciphertext = Aes256CbcEnc::new(&key.into(), &iv.into())
        .encrypt_padded_vec_mut::<Pkcs7>(&file_content);
    
    let mut output_data = Vec::new();
    output_data.extend_from_slice(&salt);
    output_data.extend_from_slice(&iv);
    output_data.extend_from_slice(&ciphertext);
    
    fs::write(output_path, &output_data)
        .map_err(|e| format!("Write error: {}", e))?;
    
    Ok(())
}

pub fn decrypt_file(password: &str, input_path: &str, output_path: &str) -> Result<(), String> {
    let encrypted_data = fs::read(input_path).map_err(|e| format!("Read error: {}", e))?;
    
    if encrypted_data.len() < SALT_LENGTH + IV_LENGTH {
        return Err("Invalid encrypted file format".to_string());
    }
    
    let salt = &encrypted_data[0..SALT_LENGTH];
    let iv = &encrypted_data[SALT_LENGTH..SALT_LENGTH + IV_LENGTH];
    let ciphertext = &encrypted_data[SALT_LENGTH + IV_LENGTH..];
    
    let key = derive_key(password, salt);
    
    let mut buffer = ciphertext.to_vec();
    let plaintext = Aes256CbcDec::new(&key.into(), iv.into())
        .decrypt_padded_vec_mut::<Pkcs7>(&mut buffer)
        .map_err(|e| format!("Decryption failed: {}", e))?;
    
    fs::write(output_path, plaintext)
        .map_err(|e| format!("Write error: {}", e))?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_roundtrip() {
        let password = "secure_password_123";
        let test_data = b"Hello, this is a secret message!";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), test_data).unwrap();
        
        encrypt_file(password, 
                    input_file.path().to_str().unwrap(),
                    encrypted_file.path().to_str().unwrap())
            .expect("Encryption should succeed");
        
        decrypt_file(password,
                    encrypted_file.path().to_str().unwrap(),
                    decrypted_file.path().to_str().unwrap())
            .expect("Decryption should succeed");
        
        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(test_data.to_vec(), decrypted_data);
    }
    
    #[test]
    fn test_wrong_password_fails() {
        let password = "correct_password";
        let wrong_password = "wrong_password";
        let test_data = b"Test data";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), test_data).unwrap();
        
        encrypt_file(password,
                    input_file.path().to_str().unwrap(),
                    encrypted_file.path().to_str().unwrap())
            .expect("Encryption should succeed");
        
        let result = decrypt_file(wrong_password,
                                 encrypted_file.path().to_str().unwrap(),
                                 decrypted_file.path().to_str().unwrap());
        
        assert!(result.is_err());
    }
}
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use std::error::Error;

pub struct FileEncryptor {
    cipher: Aes256Gcm,
}

impl FileEncryptor {
    pub fn new() -> Self {
        let key = Key::<Aes256Gcm>::generate(&mut OsRng);
        Self {
            cipher: Aes256Gcm::new(&key),
        }
    }

    pub fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
        let nonce = Nonce::generate(&mut OsRng);
        let ciphertext = self.cipher.encrypt(&nonce, plaintext)?;
        
        let mut result = Vec::with_capacity(nonce.len() + ciphertext.len());
        result.extend_from_slice(nonce.as_slice());
        result.extend_from_slice(&ciphertext);
        
        Ok(result)
    }

    pub fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
        if ciphertext.len() < 12 {
            return Err("Invalid ciphertext length".into());
        }
        
        let nonce = Nonce::from_slice(&ciphertext[..12]);
        let encrypted_data = &ciphertext[12..];
        
        let plaintext = self.cipher.decrypt(nonce, encrypted_data)?;
        Ok(plaintext)
    }
}

pub fn process_file_data(data: &[u8]) -> Result<(Vec<u8>, Vec<u8>), Box<dyn Error>> {
    let encryptor = FileEncryptor::new();
    let encrypted = encryptor.encrypt(data)?;
    let decrypted = encryptor.decrypt(&encrypted)?;
    
    Ok((encrypted, decrypted))
}