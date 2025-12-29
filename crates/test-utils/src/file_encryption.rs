
use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use hex;
use rand::RngCore;
use std::fs;
use std::io::{Read, Write};

type Aes256CbcEnc = cbc::Encryptor<aes::Aes256>;
type Aes256CbcDec = cbc::Decryptor<aes::Aes256>;

pub fn encrypt_file(input_path: &str, output_path: &str, key: &[u8; 32]) -> Result<(), String> {
    let mut file = fs::File::open(input_path).map_err(|e| format!("Failed to open input file: {}", e))?;
    let mut plaintext = Vec::new();
    file.read_to_end(&mut plaintext).map_err(|e| format!("Failed to read input file: {}", e))?;

    let mut iv = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut iv);

    let ciphertext = Aes256CbcEnc::new(key.into(), &iv.into())
        .encrypt_padded_vec_mut::<Pkcs7>(&plaintext);

    let mut output = fs::File::create(output_path).map_err(|e| format!("Failed to create output file: {}", e))?;
    output.write_all(&iv).map_err(|e| format!("Failed to write IV: {}", e))?;
    output.write_all(&ciphertext).map_err(|e| format!("Failed to write ciphertext: {}", e))?;

    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str, key: &[u8; 32]) -> Result<(), String> {
    let mut file = fs::File::open(input_path).map_err(|e| format!("Failed to open input file: {}", e))?;
    let mut encrypted_data = Vec::new();
    file.read_to_end(&mut encrypted_data).map_err(|e| format!("Failed to read input file: {}", e))?;

    if encrypted_data.len() < 16 {
        return Err("File too short to contain IV".to_string());
    }

    let iv = &encrypted_data[..16];
    let ciphertext = &encrypted_data[16..];

    let plaintext = Aes256CbcDec::new(key.into(), iv.into())
        .decrypt_padded_vec_mut::<Pkcs7>(ciphertext)
        .map_err(|e| format!("Decryption failed: {}", e))?;

    let mut output = fs::File::create(output_path).map_err(|e| format!("Failed to create output file: {}", e))?;
    output.write_all(&plaintext).map_err(|e| format!("Failed to write plaintext: {}", e))?;

    Ok(())
}

pub fn generate_key() -> [u8; 32] {
    let mut key = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut key);
    key
}

pub fn key_to_hex(key: &[u8; 32]) -> String {
    hex::encode(key)
}

pub fn hex_to_key(hex_str: &str) -> Result<[u8; 32], String> {
    let bytes = hex::decode(hex_str).map_err(|e| format!("Invalid hex string: {}", e))?;
    if bytes.len() != 32 {
        return Err("Hex string must be 64 characters (32 bytes)".to_string());
    }
    let mut key = [0u8; 32];
    key.copy_from_slice(&bytes);
    Ok(key)
}
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: u8 = 0x55;

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
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_xor_cipher_symmetry() {
        let mut data = vec![0x00, 0xFF, 0xAA, 0x55];
        let original = data.clone();
        let key = 0x33;
        
        xor_cipher(&mut data, key);
        assert_ne!(data, original);
        
        xor_cipher(&mut data, key);
        assert_eq!(data, original);
    }
    
    #[test]
    fn test_file_encryption() -> io::Result<()> {
        let content = b"Secret data that needs protection";
        let input_file = NamedTempFile::new()?;
        let output_file = NamedTempFile::new()?;
        let decrypted_file = NamedTempFile::new()?;
        
        fs::write(input_file.path(), content)?;
        
        encrypt_file(input_file.path().to_str().unwrap(), 
                    output_file.path().to_str().unwrap())?;
        
        let encrypted_content = fs::read(output_file.path())?;
        assert_ne!(encrypted_content, content);
        
        decrypt_file(output_file.path().to_str().unwrap(),
                    decrypted_file.path().to_str().unwrap())?;
        
        let decrypted_content = fs::read(decrypted_file.path())?;
        assert_eq!(decrypted_content, content);
        
        Ok(())
    }
}