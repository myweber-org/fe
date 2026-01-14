use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use std::error::Error;

pub fn encrypt_file(
    plaintext: &[u8],
    key: &[u8; 32],
) -> Result<Vec<u8>, Box<dyn Error>> {
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let nonce = Nonce::from_slice(b"unique_nonce_");
    
    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| format!("Encryption failed: {}", e))?;
    
    let mut result = Vec::with_capacity(nonce.len() + ciphertext.len());
    result.extend_from_slice(nonce);
    result.extend_from_slice(&ciphertext);
    
    Ok(result)
}

pub fn decrypt_file(
    ciphertext: &[u8],
    key: &[u8; 32],
) -> Result<Vec<u8>, Box<dyn Error>> {
    if ciphertext.len() < 12 {
        return Err("Invalid ciphertext length".into());
    }
    
    let (nonce_slice, encrypted_data) = ciphertext.split_at(12);
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let nonce = Nonce::from_slice(nonce_slice);
    
    cipher
        .decrypt(nonce, encrypted_data)
        .map_err(|e| format!("Decryption failed: {}", e).into())
}

pub fn generate_key() -> [u8; 32] {
    let mut key = [0u8; 32];
    OsRng.fill_bytes(&mut key);
    key
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_roundtrip() {
        let key = generate_key();
        let plaintext = b"Secret data that needs protection";
        
        let ciphertext = encrypt_file(plaintext, &key).unwrap();
        let decrypted = decrypt_file(&ciphertext, &key).unwrap();
        
        assert_eq!(plaintext.to_vec(), decrypted);
    }
    
    #[test]
    fn test_wrong_key_fails() {
        let key1 = generate_key();
        let mut key2 = generate_key();
        
        while key1 == key2 {
            key2 = generate_key();
        }
        
        let plaintext = b"Test data";
        let ciphertext = encrypt_file(plaintext, &key1).unwrap();
        
        assert!(decrypt_file(&ciphertext, &key2).is_err());
    }
}
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use std::error::Error;

pub fn encrypt_data(plaintext: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
    let key = Aes256Gcm::generate_key(&mut OsRng);
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(b"unique nonce");
    
    let ciphertext = cipher.encrypt(nonce, plaintext)
        .map_err(|e| format!("Encryption failed: {}", e))?;
    
    let mut result = Vec::with_capacity(key.len() + ciphertext.len());
    result.extend_from_slice(&key);
    result.extend_from_slice(&ciphertext);
    
    Ok(result)
}

pub fn decrypt_data(ciphertext: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
    if ciphertext.len() < 32 {
        return Err("Invalid ciphertext length".into());
    }
    
    let key = Key::<Aes256Gcm>::from_slice(&ciphertext[..32]);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(b"unique nonce");
    
    cipher.decrypt(nonce, &ciphertext[32..])
        .map_err(|e| format!("Decryption failed: {}", e).into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_roundtrip() {
        let original_data = b"secret message";
        let encrypted = encrypt_data(original_data).unwrap();
        let decrypted = decrypt_data(&encrypted).unwrap();
        
        assert_eq!(original_data.to_vec(), decrypted);
    }

    #[test]
    fn test_invalid_ciphertext() {
        let result = decrypt_data(b"short");
        assert!(result.is_err());
    }
}
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: u8 = 0xAA;

fn xor_cipher(data: &mut [u8], key: u8) {
    for byte in data.iter_mut() {
        *byte ^= key;
    }
}

fn process_file(input_path: &Path, output_path: &Path, key: u8) -> io::Result<()> {
    let mut file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    
    xor_cipher(&mut buffer, key);
    
    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&buffer)?;
    
    Ok(())
}

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() != 4 {
        eprintln!("Usage: {} <input_file> <output_file> <key_byte>", args[0]);
        std::process::exit(1);
    }
    
    let input_path = Path::new(&args[1]);
    let output_path = Path::new(&args[2]);
    let key = args[3].parse::<u8>().unwrap_or(DEFAULT_KEY);
    
    process_file(input_path, output_path, key)?;
    
    println!("File processed successfully with key: 0x{:02X}", key);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    
    #[test]
    fn test_xor_cipher() {
        let mut data = vec![0x00, 0xFF, 0x55, 0xAA];
        let original = data.clone();
        let key = 0xAA;
        
        xor_cipher(&mut data, key);
        xor_cipher(&mut data, key);
        
        assert_eq!(data, original);
    }
    
    #[test]
    fn test_file_processing() -> io::Result<()> {
        let test_data = b"Hello, World!";
        let input_path = Path::new("test_input.txt");
        let output_path = Path::new("test_output.txt");
        let final_path = Path::new("test_final.txt");
        
        fs::write(input_path, test_data)?;
        
        process_file(input_path, output_path, DEFAULT_KEY)?;
        process_file(output_path, final_path, DEFAULT_KEY)?;
        
        let result = fs::read(final_path)?;
        assert_eq!(result, test_data);
        
        fs::remove_file(input_path)?;
        fs::remove_file(output_path)?;
        fs::remove_file(final_path)?;
        
        Ok(())
    }
}use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHash, PasswordHasher, PasswordVerifier, SaltString
    },
    Argon2
};
use std::fs;
use std::io::{Read, Write};

const NONCE_SIZE: usize = 12;

pub struct FileEncryptor {
    cipher: Aes256Gcm,
}

impl FileEncryptor {
    pub fn new_from_password(password: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        
        let password_hash = argon2.hash_password(password.as_bytes(), &salt)?;
        let key_bytes = password_hash.hash.ok_or("Hash generation failed")?.as_bytes();
        
        let key = Key::<Aes256Gcm>::from_slice(&key_bytes[..32]);
        let cipher = Aes256Gcm::new(key);
        
        Ok(FileEncryptor { cipher })
    }

    pub fn encrypt_file(&self, input_path: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut file = fs::File::open(input_path)?;
        let mut plaintext = Vec::new();
        file.read_to_end(&mut plaintext)?;

        let nonce = Nonce::from_slice(&generate_nonce());
        let ciphertext = self.cipher.encrypt(nonce, plaintext.as_ref())
            .map_err(|e| format!("Encryption failed: {}", e))?;

        let mut output_file = fs::File::create(output_path)?;
        output_file.write_all(nonce.as_slice())?;
        output_file.write_all(&ciphertext)?;

        Ok(())
    }

    pub fn decrypt_file(&self, input_path: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut file = fs::File::open(input_path)?;
        let mut encrypted_data = Vec::new();
        file.read_to_end(&mut encrypted_data)?;

        if encrypted_data.len() < NONCE_SIZE {
            return Err("Invalid encrypted file format".into());
        }

        let (nonce_bytes, ciphertext) = encrypted_data.split_at(NONCE_SIZE);
        let nonce = Nonce::from_slice(nonce_bytes);
        
        let plaintext = self.cipher.decrypt(nonce, ciphertext)
            .map_err(|e| format!("Decryption failed: {}", e))?;

        fs::write(output_path, plaintext)?;
        Ok(())
    }
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
    fn test_encryption_decryption() {
        let password = "secure_password_123";
        let encryptor = FileEncryptor::new_from_password(password).unwrap();
        
        let test_data = b"Hello, this is a secret message!";
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), test_data).unwrap();
        
        encryptor.encrypt_file(
            input_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap()
        ).unwrap();
        
        encryptor.decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap()
        ).unwrap();
        
        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(test_data.to_vec(), decrypted_data);
    }
}