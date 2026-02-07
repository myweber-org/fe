
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: u8 = 0x55;

pub fn xor_encrypt_file(input_path: &Path, output_path: &Path, key: Option<u8>) -> io::Result<()> {
    let encryption_key = key.unwrap_or(DEFAULT_KEY);
    
    let mut input_file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    input_file.read_to_end(&mut buffer)?;
    
    let encrypted_data: Vec<u8> = buffer
        .iter()
        .map(|byte| byte ^ encryption_key)
        .collect();
    
    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&encrypted_data)?;
    
    Ok(())
}

pub fn xor_decrypt_file(input_path: &Path, output_path: &Path, key: Option<u8>) -> io::Result<()> {
    xor_encrypt_file(input_path, output_path, key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let original_text = b"Hello, Rust! This is a test message.";
        
        let mut input_file = NamedTempFile::new().unwrap();
        input_file.write_all(original_text).unwrap();
        
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        xor_encrypt_file(input_file.path(), encrypted_file.path(), Some(0x42)).unwrap();
        xor_decrypt_file(encrypted_file.path(), decrypted_file.path(), Some(0x42)).unwrap();
        
        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(original_text.to_vec(), decrypted_content);
    }
}
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

pub fn encrypt_file(input_path: &Path, output_path: &Path) -> io::Result<()> {
    let mut file = fs::File::open(input_path)?;
    let mut plaintext = Vec::new();
    file.read_to_end(&mut plaintext)?;

    let key = Aes256Gcm::generate_key(&mut OsRng);
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(b"unique_nonce_");

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_ref())
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&key)?;
    output_file.write_all(&ciphertext)?;

    Ok(())
}

pub fn decrypt_file(input_path: &Path, output_path: &Path) -> io::Result<()> {
    let mut file = fs::File::open(input_path)?;
    let mut data = Vec::new();
    file.read_to_end(&mut data)?;

    if data.len() < 32 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "File too short to contain key",
        ));
    }

    let (key_bytes, ciphertext) = data.split_at(32);
    let key = Key::<Aes256Gcm>::from_slice(key_bytes);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(b"unique_nonce_");

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&plaintext)?;

    Ok(())
}
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

pub fn xor_encrypt_file(input_path: &str, output_path: &str, key: &[u8]) -> io::Result<()> {
    let input_data = fs::read(input_path)?;
    let encrypted_data = xor_transform(&input_data, key);
    fs::write(output_path, encrypted_data)?;
    Ok(())
}

pub fn xor_decrypt_file(input_path: &str, output_path: &str, key: &[u8]) -> io::Result<()> {
    xor_encrypt_file(input_path, output_path, key)
}

fn xor_transform(data: &[u8], key: &[u8]) -> Vec<u8> {
    let key_len = key.len();
    if key_len == 0 {
        return data.to_vec();
    }
    
    data.iter()
        .enumerate()
        .map(|(i, &byte)| byte ^ key[i % key_len])
        .collect()
}

pub fn generate_random_key(length: usize) -> Vec<u8> {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    (0..length).map(|_| rng.gen()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_xor_symmetry() {
        let original = b"Hello, World!";
        let key = b"secret";
        
        let encrypted = xor_transform(original, key);
        let decrypted = xor_transform(&encrypted, key);
        
        assert_eq!(original.to_vec(), decrypted);
    }

    #[test]
    fn test_file_encryption() -> io::Result<()> {
        let temp_input = NamedTempFile::new()?;
        let temp_output = NamedTempFile::new()?;
        
        let test_data = b"Test file content";
        fs::write(temp_input.path(), test_data)?;
        
        let key = b"test_key";
        xor_encrypt_file(temp_input.path().to_str().unwrap(), 
                        temp_output.path().to_str().unwrap(), 
                        key)?;
        
        let encrypted = fs::read(temp_output.path())?;
        assert_ne!(test_data.to_vec(), encrypted);
        
        let temp_decrypted = NamedTempFile::new()?;
        xor_decrypt_file(temp_output.path().to_str().unwrap(),
                        temp_decrypted.path().to_str().unwrap(),
                        key)?;
        
        let decrypted = fs::read(temp_decrypted.path())?;
        assert_eq!(test_data.to_vec(), decrypted);
        
        Ok(())
    }
}