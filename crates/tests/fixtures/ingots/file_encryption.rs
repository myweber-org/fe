
use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use pbkdf2::pbkdf2_hmac;
use rand::RngCore;
use sha2::Sha256;
use std::fs;
use std::io::{Read, Write};

type Aes256CbcEnc = cbc::Encryptor<aes::Aes256>;
type Aes256CbcDec = cbc::Decryptor<aes::Aes256>;

const SALT_LEN: usize = 16;
const IV_LEN: usize = 16;
const KEY_ITERATIONS: u32 = 100_000;
const KEY_LEN: usize = 32;

pub struct FileCipher;

impl FileCipher {
    pub fn encrypt_file(input_path: &str, output_path: &str, password: &str) -> Result<(), String> {
        let mut file_data = fs::read(input_path).map_err(|e| format!("Read error: {}", e))?;
        
        let mut salt = [0u8; SALT_LEN];
        let mut iv = [0u8; IV_LEN];
        rand::thread_rng().fill_bytes(&mut salt);
        rand::thread_rng().fill_bytes(&mut iv);
        
        let key = Self::derive_key(password, &salt);
        
        let cipher = Aes256CbcEnc::new(&key.into(), &iv.into());
        let encrypted_data = cipher.encrypt_padded_vec_mut::<Pkcs7>(&mut file_data);
        
        let mut output = Vec::with_capacity(SALT_LEN + IV_LEN + encrypted_data.len());
        output.extend_from_slice(&salt);
        output.extend_from_slice(&iv);
        output.extend_from_slice(&encrypted_data);
        
        fs::write(output_path, &output).map_err(|e| format!("Write error: {}", e))?;
        Ok(())
    }
    
    pub fn decrypt_file(input_path: &str, output_path: &str, password: &str) -> Result<(), String> {
        let encrypted_data = fs::read(input_path).map_err(|e| format!("Read error: {}", e))?;
        
        if encrypted_data.len() < SALT_LEN + IV_LEN {
            return Err("Invalid encrypted file format".into());
        }
        
        let salt = &encrypted_data[0..SALT_LEN];
        let iv = &encrypted_data[SALT_LEN..SALT_LEN + IV_LEN];
        let ciphertext = &encrypted_data[SALT_LEN + IV_LEN..];
        
        let key = Self::derive_key(password, salt);
        
        let cipher = Aes256CbcDec::new(&key.into(), iv.into());
        let decrypted_data = cipher
            .decrypt_padded_vec_mut::<Pkcs7>(ciphertext)
            .map_err(|e| format!("Decryption failed: {}", e))?;
        
        fs::write(output_path, &decrypted_data).map_err(|e| format!("Write error: {}", e))?;
        Ok(())
    }
    
    fn derive_key(password: &str, salt: &[u8]) -> [u8; KEY_LEN] {
        let mut key = [0u8; KEY_LEN];
        pbkdf2_hmac::<Sha256>(password.as_bytes(), salt, KEY_ITERATIONS, &mut key);
        key
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_encryption_roundtrip() {
        let original_data = b"Secret data that needs protection";
        let password = "strong_password_123";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), original_data).unwrap();
        
        FileCipher::encrypt_file(
            input_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap(),
            password
        ).unwrap();
        
        FileCipher::decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap(),
            password
        ).unwrap();
        
        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(original_data.as_slice(), decrypted_data.as_slice());
    }
    
    #[test]
    fn test_wrong_password_fails() {
        let original_data = b"Test data";
        let password = "correct_password";
        let wrong_password = "wrong_password";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), original_data).unwrap();
        
        FileCipher::encrypt_file(
            input_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap(),
            password
        ).unwrap();
        
        let result = FileCipher::decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap(),
            wrong_password
        );
        
        assert!(result.is_err());
    }
}use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use std::fs;
use std::io::{self, Write};

const NONCE_SIZE: usize = 12;

pub fn encrypt_file(input_path: &str, output_path: &str, password: &str) -> io::Result<()> {
    let plaintext = fs::read(input_path)?;
    
    let key = derive_key(password);
    let cipher = Aes256Gcm::new(&key);
    
    let mut rng = OsRng;
    let nonce_bytes: [u8; NONCE_SIZE] = rng.gen();
    let nonce = Nonce::from_slice(&nonce_bytes);
    
    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_ref())
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    
    let mut output_data = Vec::with_capacity(NONCE_SIZE + ciphertext.len());
    output_data.extend_from_slice(&nonce_bytes);
    output_data.extend_from_slice(&ciphertext);
    
    fs::write(output_path, output_data)?;
    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str, password: &str) -> io::Result<()> {
    let encrypted_data = fs::read(input_path)?;
    
    if encrypted_data.len() < NONCE_SIZE {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "File too short to contain nonce",
        ));
    }
    
    let (nonce_bytes, ciphertext) = encrypted_data.split_at(NONCE_SIZE);
    let nonce = Nonce::from_slice(nonce_bytes);
    
    let key = derive_key(password);
    let cipher = Aes256Gcm::new(&key);
    
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    
    fs::write(output_path, plaintext)?;
    Ok(())
}

fn derive_key(password: &str) -> Key<Aes256Gcm> {
    let mut key_bytes = [0u8; 32];
    let password_bytes = password.as_bytes();
    
    for (i, byte) in password_bytes.iter().enumerate() {
        key_bytes[i % 32] ^= byte.wrapping_add(i as u8);
    }
    
    *Key::<Aes256Gcm>::from_slice(&key_bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_roundtrip() {
        let plaintext = b"Secret data for encryption test";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), plaintext).unwrap();
        
        let password = "strong_password_123";
        
        encrypt_file(
            input_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap(),
            password,
        )
        .unwrap();
        
        decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap(),
            password,
        )
        .unwrap();
        
        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(plaintext.to_vec(), decrypted_data);
    }
}use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: u8 = 0x55;

pub fn encrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    let encryption_key = key.unwrap_or(DEFAULT_KEY);
    
    let mut input_file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    input_file.read_to_end(&mut buffer)?;
    
    let encrypted_data: Vec<u8> = buffer
        .into_iter()
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
        
        let input_temp = NamedTempFile::new().unwrap();
        let encrypted_temp = NamedTempFile::new().unwrap();
        let decrypted_temp = NamedTempFile::new().unwrap();
        
        fs::write(input_temp.path(), original_text).unwrap();
        
        encrypt_file(
            input_temp.path().to_str().unwrap(),
            encrypted_temp.path().to_str().unwrap(),
            Some(0x42)
        ).unwrap();
        
        decrypt_file(
            encrypted_temp.path().to_str().unwrap(),
            decrypted_temp.path().to_str().unwrap(),
            Some(0x42)
        ).unwrap();
        
        let decrypted_data = fs::read(decrypted_temp.path()).unwrap();
        assert_eq!(original_text.to_vec(), decrypted_data);
    }
    
    #[test]
    fn test_default_key() {
        let test_data = b"Test data for default key";
        
        let input_temp = NamedTempFile::new().unwrap();
        let encrypted_temp = NamedTempFile::new().unwrap();
        let decrypted_temp = NamedTempFile::new().unwrap();
        
        fs::write(input_temp.path(), test_data).unwrap();
        
        encrypt_file(
            input_temp.path().to_str().unwrap(),
            encrypted_temp.path().to_str().unwrap(),
            None
        ).unwrap();
        
        decrypt_file(
            encrypted_temp.path().to_str().unwrap(),
            decrypted_temp.path().to_str().unwrap(),
            None
        ).unwrap();
        
        let result = fs::read(decrypted_temp.path()).unwrap();
        assert_eq!(test_data.to_vec(), result);
    }
}
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: u8 = 0x55;

pub fn encrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    let encryption_key = key.unwrap_or(DEFAULT_KEY);
    
    let mut input_file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    input_file.read_to_end(&mut buffer)?;
    
    for byte in buffer.iter_mut() {
        *byte ^= encryption_key;
    }
    
    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&buffer)?;
    
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
        let original_text = b"Hello, Rust encryption!";
        let test_key = 0xAA;
        
        let input_temp = NamedTempFile::new().unwrap();
        let encrypted_temp = NamedTempFile::new().unwrap();
        let decrypted_temp = NamedTempFile::new().unwrap();
        
        fs::write(input_temp.path(), original_text).unwrap();
        
        encrypt_file(
            input_temp.path().to_str().unwrap(),
            encrypted_temp.path().to_str().unwrap(),
            Some(test_key)
        ).unwrap();
        
        let encrypted_content = fs::read(encrypted_temp.path()).unwrap();
        assert_ne!(encrypted_content, original_text);
        
        decrypt_file(
            encrypted_temp.path().to_str().unwrap(),
            decrypted_temp.path().to_str().unwrap(),
            Some(test_key)
        ).unwrap();
        
        let decrypted_content = fs::read(decrypted_temp.path()).unwrap();
        assert_eq!(decrypted_content, original_text);
    }
    
    #[test]
    fn test_default_key() {
        let original_text = b"Test with default key";
        
        let input_temp = NamedTempFile::new().unwrap();
        let encrypted_temp = NamedTempFile::new().unwrap();
        let decrypted_temp = NamedTempFile::new().unwrap();
        
        fs::write(input_temp.path(), original_text).unwrap();
        
        encrypt_file(
            input_temp.path().to_str().unwrap(),
            encrypted_temp.path().to_str().unwrap(),
            None
        ).unwrap();
        
        decrypt_file(
            encrypted_temp.path().to_str().unwrap(),
            decrypted_temp.path().to_str().unwrap(),
            None
        ).unwrap();
        
        let decrypted_content = fs::read(decrypted_temp.path()).unwrap();
        assert_eq!(decrypted_content, original_text);
    }
}
use std::fs;
use std::io::{self, Read, Write};

pub fn xor_encrypt(data: &[u8], key: &[u8]) -> Vec<u8> {
    data.iter()
        .enumerate()
        .map(|(i, &byte)| byte ^ key[i % key.len()])
        .collect()
}

pub fn process_file(input_path: &str, output_path: &str, key: &[u8]) -> io::Result<()> {
    let mut input_file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    input_file.read_to_end(&mut buffer)?;

    let processed_data = xor_encrypt(&buffer, key);

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&processed_data)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xor_encrypt() {
        let data = b"Hello, World!";
        let key = b"secret";
        let encrypted = xor_encrypt(data, key);
        let decrypted = xor_encrypt(&encrypted, key);
        
        assert_eq!(data.to_vec(), decrypted);
    }

    #[test]
    fn test_empty_data() {
        let data = b"";
        let key = b"key";
        let result = xor_encrypt(data, key);
        assert!(result.is_empty());
    }
}