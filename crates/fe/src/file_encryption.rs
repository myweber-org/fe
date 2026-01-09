use std::fs;
use std::io::{self, Read, Write};

fn xor_encrypt_decrypt(data: &[u8], key: &[u8]) -> Vec<u8> {
    data.iter()
        .enumerate()
        .map(|(i, &byte)| byte ^ key[i % key.len()])
        .collect()
}

fn process_file(input_path: &str, output_path: &str, key: &[u8]) -> io::Result<()> {
    let mut input_file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    input_file.read_to_end(&mut buffer)?;

    let processed_data = xor_encrypt_decrypt(&buffer, key);

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&processed_data)?;

    Ok(())
}

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 4 {
        eprintln!("Usage: {} <input> <output> <key>", args[0]);
        std::process::exit(1);
    }

    let input_path = &args[1];
    let output_path = &args[2];
    let key = args[3].as_bytes();

    process_file(input_path, output_path, key)?;
    println!("File processed successfully: {} -> {}", input_path, output_path);

    Ok(())
}use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

/// XOR cipher implementation for file encryption/decryption
pub struct XorCipher {
    key: Vec<u8>,
}

impl XorCipher {
    /// Create a new cipher with the given key
    pub fn new(key: &[u8]) -> Self {
        XorCipher { key: key.to_vec() }
    }

    /// Process data using XOR cipher
    pub fn process(&self, data: &[u8]) -> Vec<u8> {
        data.iter()
            .enumerate()
            .map(|(i, &byte)| byte ^ self.key[i % self.key.len()])
            .collect()
    }
}

/// Encrypt or decrypt a file using XOR cipher
pub fn process_file(input_path: &Path, output_path: &Path, key: &[u8]) -> io::Result<()> {
    let cipher = XorCipher::new(key);
    
    let mut input_file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    input_file.read_to_end(&mut buffer)?;
    
    let processed_data = cipher.process(&buffer);
    
    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&processed_data)?;
    
    Ok(())
}

/// Generate a random key of specified length
pub fn generate_key(length: usize) -> Vec<u8> {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    (0..length).map(|_| rng.gen()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;

    #[test]
    fn test_xor_cipher_symmetry() {
        let key = b"secret_key";
        let cipher = XorCipher::new(key);
        let original_data = b"Hello, World! This is a test message.";
        
        let encrypted = cipher.process(original_data);
        let decrypted = cipher.process(&encrypted);
        
        assert_eq!(original_data.to_vec(), decrypted);
    }

    #[test]
    fn test_file_processing() -> io::Result<()> {
        let key = b"test_key_123";
        let original_content = b"Sample file content for encryption test.";
        
        let input_file = NamedTempFile::new()?;
        let output_file = NamedTempFile::new()?;
        
        fs::write(input_file.path(), original_content)?;
        
        process_file(input_file.path(), output_file.path(), key)?;
        
        let encrypted_content = fs::read(output_file.path())?;
        assert_ne!(original_content, encrypted_content.as_slice());
        
        let cipher = XorCipher::new(key);
        let decrypted_content = cipher.process(&encrypted_content);
        assert_eq!(original_content.to_vec(), decrypted_content);
        
        Ok(())
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

pub fn encrypt_file(input_path: &str, output_path: &str) -> io::Result<()> {
    let key = DEFAULT_KEY;
    process_file(Path::new(input_path), Path::new(output_path), key)
}

pub fn decrypt_file(input_path: &str, output_path: &str) -> io::Result<()> {
    let key = DEFAULT_KEY;
    process_file(Path::new(input_path), Path::new(output_path), key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
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
        let content = b"Secret data to encrypt";
        let input_file = NamedTempFile::new()?;
        let output_file = NamedTempFile::new()?;
        let decrypted_file = NamedTempFile::new()?;

        fs::write(input_file.path(), content)?;

        encrypt_file(
            input_file.path().to_str().unwrap(),
            output_file.path().to_str().unwrap(),
        )?;

        let encrypted_content = fs::read(output_file.path())?;
        assert_ne!(encrypted_content, content);

        decrypt_file(
            output_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap(),
        )?;

        let decrypted_content = fs::read(decrypted_file.path())?;
        assert_eq!(decrypted_content, content);

        Ok(())
    }
}use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use hex::{decode, encode};
use rand::RngCore;
use std::fs;
use std::io::{Read, Write};

type Aes256CbcEnc = cbc::Encryptor<aes::Aes256>;
type Aes256CbcDec = cbc::Decryptor<aes::Aes256>;

const KEY_LENGTH: usize = 32;
const IV_LENGTH: usize = 16;

pub fn generate_key() -> Vec<u8> {
    let mut key = vec![0u8; KEY_LENGTH];
    rand::thread_rng().fill_bytes(&mut key);
    key
}

pub fn encrypt_file(input_path: &str, output_path: &str, key: &[u8]) -> Result<(), String> {
    if key.len() != KEY_LENGTH {
        return Err(format!("Key must be {} bytes", KEY_LENGTH));
    }

    let mut file = fs::File::open(input_path)
        .map_err(|e| format!("Failed to open input file: {}", e))?;
    let mut plaintext = Vec::new();
    file.read_to_end(&mut plaintext)
        .map_err(|e| format!("Failed to read input file: {}", e))?;

    let mut iv = [0u8; IV_LENGTH];
    rand::thread_rng().fill_bytes(&mut iv);

    let ciphertext = Aes256CbcEnc::new(key.into(), &iv.into())
        .encrypt_padded_vec_mut::<Pkcs7>(&plaintext);

    let mut output = fs::File::create(output_path)
        .map_err(|e| format!("Failed to create output file: {}", e))?;

    output.write_all(&iv).map_err(|e| format!("Failed to write IV: {}", e))?;
    output.write_all(&ciphertext).map_err(|e| format!("Failed to write ciphertext: {}", e))?;

    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str, key: &[u8]) -> Result<(), String> {
    if key.len() != KEY_LENGTH {
        return Err(format!("Key must be {} bytes", KEY_LENGTH));
    }

    let mut file = fs::File::open(input_path)
        .map_err(|e| format!("Failed to open input file: {}", e))?;
    let mut encrypted_data = Vec::new();
    file.read_to_end(&mut encrypted_data)
        .map_err(|e| format!("Failed to read input file: {}", e))?;

    if encrypted_data.len() < IV_LENGTH {
        return Err("File too short to contain IV".to_string());
    }

    let (iv, ciphertext) = encrypted_data.split_at(IV_LENGTH);
    let iv_array: [u8; IV_LENGTH] = iv.try_into().unwrap();

    let plaintext = Aes256CbcDec::new(key.into(), &iv_array.into())
        .decrypt_padded_vec_mut::<Pkcs7>(ciphertext)
        .map_err(|e| format!("Decryption failed: {}", e))?;

    let mut output = fs::File::create(output_path)
        .map_err(|e| format!("Failed to create output file: {}", e))?;
    output.write_all(&plaintext)
        .map_err(|e| format!("Failed to write plaintext: {}", e))?;

    Ok(())
}

pub fn key_to_hex(key: &[u8]) -> String {
    encode(key)
}

pub fn hex_to_key(hex_str: &str) -> Result<Vec<u8>, String> {
    decode(hex_str).map_err(|e| format!("Invalid hex string: {}", e))
}