use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use std::error::Error;

pub fn encrypt_data(plaintext: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
    let key = Aes256Gcm::generate_key(&mut OsRng);
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(b"unique_nonce_");
    
    let ciphertext = cipher.encrypt(nonce, plaintext)
        .map_err(|e| format!("Encryption failed: {}", e))?;
    
    let mut result = Vec::with_capacity(key.len() + nonce.len() + ciphertext.len());
    result.extend_from_slice(&key);
    result.extend_from_slice(nonce);
    result.extend_from_slice(&ciphertext);
    
    Ok(result)
}

pub fn decrypt_data(encrypted_data: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
    if encrypted_data.len() < 48 {
        return Err("Invalid encrypted data length".into());
    }
    
    let key = Key::<Aes256Gcm>::from_slice(&encrypted_data[0..32]);
    let nonce = Nonce::from_slice(&encrypted_data[32..44]);
    let ciphertext = &encrypted_data[44..];
    
    let cipher = Aes256Gcm::new(key);
    let plaintext = cipher.decrypt(nonce, ciphertext)
        .map_err(|e| format!("Decryption failed: {}", e))?;
    
    Ok(plaintext)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_roundtrip() {
        let original_data = b"Secret message for encryption test";
        let encrypted = encrypt_data(original_data).unwrap();
        let decrypted = decrypt_data(&encrypted).unwrap();
        
        assert_eq!(original_data.to_vec(), decrypted);
    }

    #[test]
    fn test_invalid_data() {
        let invalid_data = b"too_short";
        let result = decrypt_data(invalid_data);
        assert!(result.is_err());
    }
}
use std::fs;
use std::io::{self, Read, Write};

pub fn xor_encrypt_file(input_path: &str, output_path: &str, key: &[u8]) -> io::Result<()> {
    let mut input_file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    input_file.read_to_end(&mut buffer)?;

    let encrypted_data: Vec<u8> = buffer
        .iter()
        .enumerate()
        .map(|(i, &byte)| byte ^ key[i % key.len()])
        .collect();

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&encrypted_data)?;

    Ok(())
}

pub fn xor_decrypt_file(input_path: &str, output_path: &str, key: &[u8]) -> io::Result<()> {
    xor_encrypt_file(input_path, output_path, key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_xor_encryption() {
        let test_data = b"Hello, Rust!";
        let key = b"secret";
        let input_file = "test_input.txt";
        let encrypted_file = "test_encrypted.txt";
        let decrypted_file = "test_decrypted.txt";

        fs::write(input_file, test_data).unwrap();

        xor_encrypt_file(input_file, encrypted_file, key).unwrap();
        xor_decrypt_file(encrypted_file, decrypted_file, key).unwrap();

        let decrypted_data = fs::read(decrypted_file).unwrap();
        assert_eq!(decrypted_data, test_data);

        fs::remove_file(input_file).unwrap_or_default();
        fs::remove_file(encrypted_file).unwrap_or_default();
        fs::remove_file(decrypted_file).unwrap_or_default();
    }
}
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const NONCE_SIZE: usize = 12;
const SALT_SIZE: usize = 16;

pub struct FileEncryptor {
    cipher: Aes256Gcm,
}

impl FileEncryptor {
    pub fn new(password: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2.hash_password(password.as_bytes(), &salt)?;
        let key_bytes = password_hash.hash.ok_or("Hash generation failed")?;
        let key = Key::<Aes256Gcm>::from_slice(key_bytes.as_bytes());
        let cipher = Aes256Gcm::new(key);
        Ok(Self { cipher })
    }

    pub fn encrypt_file(
        &self,
        input_path: &Path,
        output_path: &Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut file = fs::File::open(input_path)?;
        let mut plaintext = Vec::new();
        file.read_to_end(&mut plaintext)?;

        let mut nonce_bytes = [0u8; NONCE_SIZE];
        OsRng.fill(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = self
            .cipher
            .encrypt(nonce, plaintext.as_ref())
            .map_err(|e| format!("Encryption failed: {}", e))?;

        let mut output = fs::File::create(output_path)?;
        output.write_all(&nonce_bytes)?;
        output.write_all(&ciphertext)?;

        Ok(())
    }

    pub fn decrypt_file(
        &self,
        input_path: &Path,
        output_path: &Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut file = fs::File::open(input_path)?;
        let mut encrypted_data = Vec::new();
        file.read_to_end(&mut encrypted_data)?;

        if encrypted_data.len() < NONCE_SIZE {
            return Err("Invalid encrypted file format".into());
        }

        let (nonce_bytes, ciphertext) = encrypted_data.split_at(NONCE_SIZE);
        let nonce = Nonce::from_slice(nonce_bytes);

        let plaintext = self
            .cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| format!("Decryption failed: {}", e))?;

        fs::write(output_path, plaintext)?;
        Ok(())
    }
}

pub fn process_encryption() -> Result<(), Box<dyn std::error::Error>> {
    let password = "secure_password_123";
    let encryptor = FileEncryptor::new(password)?;

    let test_data = b"Confidential data: API keys, tokens, and secrets";
    let input_path = Path::new("test_input.txt");
    let encrypted_path = Path::new("test_encrypted.bin");
    let decrypted_path = Path::new("test_decrypted.txt");

    fs::write(input_path, test_data)?;

    encryptor.encrypt_file(input_path, encrypted_path)?;
    println!("File encrypted successfully");

    encryptor.decrypt_file(encrypted_path, decrypted_path)?;
    println!("File decrypted successfully");

    let decrypted_content = fs::read(decrypted_path)?;
    assert_eq!(test_data.as_ref(), decrypted_content);
    println!("Decryption verified successfully");

    fs::remove_file(input_path)?;
    fs::remove_file(encrypted_path)?;
    fs::remove_file(decrypted_path)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_roundtrip() {
        let result = process_encryption();
        assert!(result.is_ok());
    }
}