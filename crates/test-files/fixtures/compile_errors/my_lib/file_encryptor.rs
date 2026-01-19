
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: u8 = 0xAA;

pub fn encrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
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

pub fn decrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    encrypt_file(input_path, output_path, key)
}

pub fn process_files() -> io::Result<()> {
    let test_data = b"Secret message for encryption test";
    let test_file = "test_input.bin";
    let encrypted_file = "test_encrypted.bin";
    let decrypted_file = "test_decrypted.bin";
    
    fs::write(test_file, test_data)?;
    
    println!("Encrypting file...");
    encrypt_file(test_file, encrypted_file, Some(0xCC))?;
    
    println!("Decrypting file...");
    decrypt_file(encrypted_file, decrypted_file, Some(0xCC))?;
    
    let decrypted_content = fs::read(decrypted_file)?;
    
    if decrypted_content == test_data {
        println!("Encryption/decryption successful!");
    } else {
        println!("Encryption/decryption failed!");
    }
    
    fs::remove_file(test_file)?;
    fs::remove_file(encrypted_file)?;
    fs::remove_file(decrypted_file)?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_xor_cipher() {
        let data = vec![0x00, 0xFF, 0x55, 0xAA];
        let key = 0xCC;
        
        let encrypted: Vec<u8> = data.iter().map(|byte| byte ^ key).collect();
        let decrypted: Vec<u8> = encrypted.iter().map(|byte| byte ^ key).collect();
        
        assert_eq!(data, decrypted);
    }
}use base64::{engine::general_purpose, Engine as _};
use std::fs;
use std::io::{self, Read, Write};

const BUFFER_SIZE: usize = 8192;

pub fn encrypt_file(input_path: &str, output_path: &str, key: &[u8]) -> io::Result<()> {
    let mut input_file = fs::File::open(input_path)?;
    let mut output_file = fs::File::create(output_path)?;
    
    let mut buffer = [0u8; BUFFER_SIZE];
    let key_len = key.len();
    let mut key_index = 0;
    
    loop {
        let bytes_read = input_file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        
        for i in 0..bytes_read {
            buffer[i] ^= key[key_index];
            key_index = (key_index + 1) % key_len;
        }
        
        let encoded = general_purpose::STANDARD.encode(&buffer[..bytes_read]);
        output_file.write_all(encoded.as_bytes())?;
        output_file.write_all(b"\n")?;
    }
    
    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str, key: &[u8]) -> io::Result<()> {
    let input_file = fs::File::open(input_path)?;
    let mut output_file = fs::File::create(output_path)?;
    let reader = io::BufReader::new(input_file);
    
    let key_len = key.len();
    let mut key_index = 0;
    
    for line in io::BufRead::lines(reader) {
        let line = line?;
        let decoded = general_purpose::STANDARD.decode(line.as_bytes())
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        
        let mut decrypted = Vec::with_capacity(decoded.len());
        for byte in decoded {
            decrypted.push(byte ^ key[key_index]);
            key_index = (key_index + 1) % key_len;
        }
        
        output_file.write_all(&decrypted)?;
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_encrypt_decrypt() {
        let original_content = b"Hello, World! This is a test message.";
        let key = b"secret_key";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), original_content).unwrap();
        
        encrypt_file(
            input_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap(),
            key
        ).unwrap();
        
        decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap(),
            key
        ).unwrap();
        
        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(original_content.to_vec(), decrypted_content);
    }
}