
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
        let original_text = b"Hello, Rust!";
        let test_key = Some(0x55);
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), original_text).unwrap();
        
        encrypt_file(
            input_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap(),
            test_key
        ).unwrap();
        
        let encrypted_content = fs::read(encrypted_file.path()).unwrap();
        assert_ne!(encrypted_content, original_text);
        
        decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap(),
            test_key
        ).unwrap();
        
        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(decrypted_content, original_text);
    }
    
    #[test]
    fn test_default_key() {
        let original_text = b"Test with default key";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), original_text).unwrap();
        
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
        assert_eq!(decrypted_content, original_text);
    }
}
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
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
    
    let ciphertext = cipher.encrypt(nonce, plaintext.as_ref())
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    
    let mut output_data = Vec::with_capacity(NONCE_SIZE + ciphertext.len());
    output_data.extend_from_slice(&nonce_bytes);
    output_data.extend_from_slice(&ciphertext);
    
    fs::write(output_path, output_data)
}

pub fn decrypt_file(input_path: &str, output_path: &str, password: &str) -> io::Result<()> {
    let encrypted_data = fs::read(input_path)?;
    
    if encrypted_data.len() < NONCE_SIZE {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "File too short"));
    }
    
    let (nonce_bytes, ciphertext) = encrypted_data.split_at(NONCE_SIZE);
    let nonce = Nonce::from_slice(nonce_bytes);
    
    let key = derive_key(password);
    let cipher = Aes256Gcm::new(&key);
    
    let plaintext = cipher.decrypt(nonce, ciphertext)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    
    fs::write(output_path, plaintext)
}

fn derive_key(password: &str) -> Key<Aes256Gcm> {
    let mut key_bytes = [0u8; 32];
    let password_bytes = password.as_bytes();
    
    for (i, &byte) in password_bytes.iter().enumerate() {
        key_bytes[i % 32] ^= byte;
    }
    
    Key::<Aes256Gcm>::from_slice(&key_bytes).clone()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_roundtrip() {
        let plaintext = b"Secret data that needs protection";
        let password = "strong_password_123";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), plaintext).unwrap();
        
        encrypt_file(
            input_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap(),
            password
        ).unwrap();
        
        decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap(),
            password
        ).unwrap();
        
        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(plaintext.to_vec(), decrypted_data);
    }
}
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: u8 = 0xAA;

fn xor_cipher(data: &mut [u8], key: u8) {
    for byte in data {
        *byte ^= key;
    }
}

fn process_file(input_path: &str, output_path: &str, key: u8) -> io::Result<()> {
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
        eprintln!("Usage: {} <encrypt|decrypt> <input_file> <output_file>", args[0]);
        std::process::exit(1);
    }

    let mode = &args[1];
    let input_file = &args[2];
    let output_file = &args[3];

    if !Path::new(input_file).exists() {
        eprintln!("Error: Input file '{}' does not exist", input_file);
        std::process::exit(1);
    }

    let key = match std::env::var("ENCRYPTION_KEY") {
        Ok(val) => val.parse().unwrap_or(DEFAULT_KEY),
        Err(_) => DEFAULT_KEY,
    };

    match mode.as_str() {
        "encrypt" | "decrypt" => process_file(input_file, output_file, key),
        _ => {
            eprintln!("Error: Mode must be 'encrypt' or 'decrypt'");
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_xor_cipher() {
        let mut data = vec![0x00, 0xFF, 0x55, 0xAA];
        let original = data.clone();
        let key = 0xAA;

        xor_cipher(&mut data, key);
        assert_ne!(data, original);

        xor_cipher(&mut data, key);
        assert_eq!(data, original);
    }

    #[test]
    fn test_file_encryption() -> io::Result<()> {
        let mut temp_input = NamedTempFile::new()?;
        let test_data = b"Hello, World!";
        temp_input.write_all(test_data)?;

        let input_path = temp_input.path().to_str().unwrap();
        let temp_output = NamedTempFile::new()?;
        let output_path = temp_output.path().to_str().unwrap();

        process_file(input_path, output_path, DEFAULT_KEY)?;

        let encrypted = fs::read(output_path)?;
        assert_ne!(encrypted, test_data);

        let temp_decrypted = NamedTempFile::new()?;
        let decrypted_path = temp_decrypted.path().to_str().unwrap();
        process_file(output_path, decrypted_path, DEFAULT_KEY)?;

        let decrypted = fs::read(decrypted_path)?;
        assert_eq!(decrypted, test_data);

        Ok(())
    }
}
use aes::Aes256;
use block_modes::{BlockMode, Cbc};
use block_modes::block_padding::Pkcs7;
use rand::RngCore;
use std::fs;
use std::io::{Read, Write};
use std::path::Path;

type Aes256Cbc = Cbc<Aes256, Pkcs7>;

const KEY_SIZE: usize = 32;
const IV_SIZE: usize = 16;

pub struct EncryptionResult {
    pub iv: Vec<u8>,
    pub ciphertext: Vec<u8>,
}

pub fn generate_key() -> Vec<u8> {
    let mut key = vec![0u8; KEY_SIZE];
    rand::thread_rng().fill_bytes(&mut key);
    key
}

pub fn encrypt_file(key: &[u8], input_path: &Path, output_path: &Path) -> Result<EncryptionResult, String> {
    if key.len() != KEY_SIZE {
        return Err(format!("Key must be {} bytes", KEY_SIZE));
    }

    let mut input_file = fs::File::open(input_path)
        .map_err(|e| format!("Failed to open input file: {}", e))?;
    
    let mut plaintext = Vec::new();
    input_file.read_to_end(&mut plaintext)
        .map_err(|e| format!("Failed to read input file: {}", e))?;

    let mut iv = vec![0u8; IV_SIZE];
    rand::thread_rng().fill_bytes(&mut iv);

    let cipher = Aes256Cbc::new_from_slices(key, &iv)
        .map_err(|e| format!("Failed to create cipher: {}", e))?;

    let ciphertext = cipher.encrypt_vec(&plaintext);

    let mut output_file = fs::File::create(output_path)
        .map_err(|e| format!("Failed to create output file: {}", e))?;
    
    output_file.write_all(&ciphertext)
        .map_err(|e| format!("Failed to write output file: {}", e))?;

    Ok(EncryptionResult {
        iv,
        ciphertext,
    })
}

pub fn decrypt_file(key: &[u8], iv: &[u8], input_path: &Path, output_path: &Path) -> Result<Vec<u8>, String> {
    if key.len() != KEY_SIZE {
        return Err(format!("Key must be {} bytes", KEY_SIZE));
    }
    
    if iv.len() != IV_SIZE {
        return Err(format!("IV must be {} bytes", IV_SIZE));
    }

    let mut input_file = fs::File::open(input_path)
        .map_err(|e| format!("Failed to open input file: {}", e))?;
    
    let mut ciphertext = Vec::new();
    input_file.read_to_end(&mut ciphertext)
        .map_err(|e| format!("Failed to read input file: {}", e))?;

    let cipher = Aes256Cbc::new_from_slices(key, iv)
        .map_err(|e| format!("Failed to create cipher: {}", e))?;

    let plaintext = cipher.decrypt_vec(&ciphertext)
        .map_err(|e| format!("Decryption failed: {}", e))?;

    let mut output_file = fs::File::create(output_path)
        .map_err(|e| format!("Failed to create output file: {}", e))?;
    
    output_file.write_all(&plaintext)
        .map_err(|e| format!("Failed to write output file: {}", e))?;

    Ok(plaintext)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let key = generate_key();
        let test_data = b"Hello, this is a secret message!";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        fs::write(input_file.path(), test_data).unwrap();

        let enc_result = encrypt_file(&key, input_file.path(), encrypted_file.path()).unwrap();
        
        let decrypted = decrypt_file(&key, &enc_result.iv, encrypted_file.path(), decrypted_file.path()).unwrap();
        
        assert_eq!(test_data.to_vec(), decrypted);
    }
}