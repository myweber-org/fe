use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use std::fs::{self, File};
use std::io::{Read, Write};

const NONCE_SIZE: usize = 12;

pub fn encrypt_file(input_path: &str, output_path: &str, password: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::open(input_path)?;
    let mut plaintext = Vec::new();
    file.read_to_end(&mut plaintext)?;

    let key = derive_key(password);
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(&generate_nonce());

    let ciphertext = cipher.encrypt(nonce, plaintext.as_ref())
        .map_err(|e| format!("Encryption failed: {}", e))?;

    let mut output = File::create(output_path)?;
    output.write_all(nonce.as_slice())?;
    output.write_all(&ciphertext)?;

    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str, password: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::open(input_path)?;
    let mut data = Vec::new();
    file.read_to_end(&mut data)?;

    if data.len() < NONCE_SIZE {
        return Err("File too short to contain nonce".into());
    }

    let (nonce_slice, ciphertext) = data.split_at(NONCE_SIZE);
    let nonce = Nonce::from_slice(nonce_slice);
    let key = derive_key(password);
    let cipher = Aes256Gcm::new(&key);

    let plaintext = cipher.decrypt(nonce, ciphertext)
        .map_err(|e| format!("Decryption failed: {}", e))?;

    let mut output = File::create(output_path)?;
    output.write_all(&plaintext)?;

    Ok(())
}

fn derive_key(password: &str) -> Key<Aes256Gcm> {
    let mut key = [0u8; 32];
    let bytes = password.as_bytes();
    for (i, &byte) in bytes.iter().cycle().take(32).enumerate() {
        key[i] = byte.wrapping_add(i as u8);
    }
    *Key::<Aes256Gcm>::from_slice(&key)
}

fn generate_nonce() -> [u8; NONCE_SIZE] {
    let mut nonce = [0u8; NONCE_SIZE];
    OsRng.fill_bytes(&mut nonce);
    nonce
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_roundtrip() {
        let plaintext = b"Secret data for encryption test";
        let password = "strong_password_123";

        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        fs::write(input_file.path(), plaintext).unwrap();

        encrypt_file(input_file.path().to_str().unwrap(),
                    encrypted_file.path().to_str().unwrap(),
                    password).unwrap();

        decrypt_file(encrypted_file.path().to_str().unwrap(),
                    decrypted_file.path().to_str().unwrap(),
                    password).unwrap();

        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(plaintext.to_vec(), decrypted_data);
    }
}use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

/// XOR-based file encryption/decryption utility
pub struct FileCipher {
    key: Vec<u8>,
}

impl FileCipher {
    /// Create a new cipher with the given key
    pub fn new(key: &str) -> Self {
        FileCipher {
            key: key.as_bytes().to_vec(),
        }
    }

    /// Encrypt or decrypt a file using XOR cipher
    pub fn process_file(&self, input_path: &Path, output_path: &Path) -> io::Result<()> {
        let mut input_file = fs::File::open(input_path)?;
        let mut buffer = Vec::new();
        input_file.read_to_end(&mut buffer)?;

        let processed_data = self.xor_transform(&buffer);

        let mut output_file = fs::File::create(output_path)?;
        output_file.write_all(&processed_data)?;

        Ok(())
    }

    /// Apply XOR transformation to data
    fn xor_transform(&self, data: &[u8]) -> Vec<u8> {
        let key_len = self.key.len();
        if key_len == 0 {
            return data.to_vec();
        }

        data.iter()
            .enumerate()
            .map(|(i, &byte)| byte ^ self.key[i % key_len])
            .collect()
    }

    /// Generate a simple checksum for verification
    pub fn calculate_checksum(data: &[u8]) -> u32 {
        data.iter().fold(0u32, |acc, &byte| {
            acc.wrapping_add(byte as u32)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_xor_symmetry() {
        let cipher = FileCipher::new("secret_key");
        let original_data = b"Hello, World! This is test data.";
        
        let encrypted = cipher.xor_transform(original_data);
        let decrypted = cipher.xor_transform(&encrypted);
        
        assert_eq!(original_data, decrypted.as_slice());
    }

    #[test]
    fn test_file_processing() -> io::Result<()> {
        let cipher = FileCipher::new("test_key_123");
        
        let mut input_file = NamedTempFile::new()?;
        let test_content = b"Sample file content for encryption test";
        input_file.write_all(test_content)?;
        
        let output_file = NamedTempFile::new()?;
        
        cipher.process_file(input_file.path(), output_file.path())?;
        
        let mut processed_content = Vec::new();
        fs::File::open(output_file.path())?.read_to_end(&mut processed_content)?;
        
        assert_ne!(test_content, processed_content.as_slice());
        
        let restored = cipher.xor_transform(&processed_content);
        assert_eq!(test_content, restored.as_slice());
        
        Ok(())
    }

    #[test]
    fn test_checksum() {
        let data1 = b"identical data";
        let data2 = b"identical data";
        let data3 = b"different data";
        
        let sum1 = FileCipher::calculate_checksum(data1);
        let sum2 = FileCipher::calculate_checksum(data2);
        let sum3 = FileCipher::calculate_checksum(data3);
        
        assert_eq!(sum1, sum2);
        assert_ne!(sum1, sum3);
    }
}use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

pub fn xor_encrypt_file(input_path: &str, output_path: &str, key: &[u8]) -> io::Result<()> {
    let input_data = fs::read(input_path)?;
    let encrypted_data: Vec<u8> = input_data
        .iter()
        .enumerate()
        .map(|(i, &byte)| byte ^ key[i % key.len()])
        .collect();
    
    fs::write(output_path, encrypted_data)?;
    Ok(())
}

pub fn xor_decrypt_file(input_path: &str, output_path: &str, key: &[u8]) -> io::Result<()> {
    xor_encrypt_file(input_path, output_path, key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let key = b"secret_key";
        let original_content = b"Hello, this is a test message!";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), original_content).unwrap();
        
        xor_encrypt_file(
            input_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap(),
            key
        ).unwrap();
        
        xor_decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap(),
            key
        ).unwrap();
        
        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(original_content.to_vec(), decrypted_content);
    }
}