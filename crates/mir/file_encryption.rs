
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: u8 = 0x55;

fn xor_cipher(data: &mut [u8], key: u8) {
    for byte in data.iter_mut() {
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
    
    if args.len() != 3 {
        eprintln!("Usage: {} <input_file> <output_file>", args[0]);
        std::process::exit(1);
    }
    
    let input_path = &args[1];
    let output_path = &args[2];
    
    if !Path::new(input_path).exists() {
        eprintln!("Error: Input file '{}' does not exist", input_path);
        std::process::exit(1);
    }
    
    match process_file(input_path, output_path, DEFAULT_KEY) {
        Ok(_) => println!("File processed successfully"),
        Err(e) => eprintln!("Error processing file: {}", e),
    }
    
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
        let key = 0x55;
        
        xor_cipher(&mut data, key);
        assert_ne!(data, original);
        
        xor_cipher(&mut data, key);
        assert_eq!(data, original);
    }
    
    #[test]
    fn test_file_encryption() {
        let test_data = b"Hello, World!";
        let input_path = "test_input.txt";
        let output_path = "test_output.txt";
        
        fs::write(input_path, test_data).unwrap();
        
        process_file(input_path, output_path, DEFAULT_KEY).unwrap();
        let encrypted = fs::read(output_path).unwrap();
        assert_ne!(encrypted, test_data);
        
        process_file(output_path, "test_decrypted.txt", DEFAULT_KEY).unwrap();
        let decrypted = fs::read("test_decrypted.txt").unwrap();
        assert_eq!(decrypted, test_data);
        
        fs::remove_file(input_path).unwrap();
        fs::remove_file(output_path).unwrap();
        fs::remove_file("test_decrypted.txt").unwrap();
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
        let temp_input = "test_input.txt";
        let temp_encrypted = "test_encrypted.txt";
        let temp_decrypted = "test_decrypted.txt";

        fs::write(temp_input, test_data).unwrap();

        xor_encrypt_file(temp_input, temp_encrypted, key).unwrap();
        xor_decrypt_file(temp_encrypted, temp_decrypted, key).unwrap();

        let decrypted_data = fs::read(temp_decrypted).unwrap();
        assert_eq!(decrypted_data, test_data);

        fs::remove_file(temp_input).unwrap_or_default();
        fs::remove_file(temp_encrypted).unwrap_or_default();
        fs::remove_file(temp_decrypted).unwrap_or_default();
    }
}
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use std::fs;
use std::io::{self, Write};

const NONCE_SIZE: usize = 12;

pub fn encrypt_file(input_path: &str, output_path: &str, key: &[u8; 32]) -> io::Result<()> {
    let cipher = Aes256Gcm::new_from_slice(key).unwrap();
    let nonce = Nonce::from_slice(&OsRng.gen::<[u8; NONCE_SIZE]>());

    let plaintext = fs::read(input_path)?;
    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_ref())
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let mut output_data = Vec::with_capacity(NONCE_SIZE + ciphertext.len());
    output_data.extend_from_slice(nonce);
    output_data.extend_from_slice(&ciphertext);

    fs::write(output_path, output_data)
}

pub fn decrypt_file(input_path: &str, output_path: &str, key: &[u8; 32]) -> io::Result<()> {
    let cipher = Aes256Gcm::new_from_slice(key).unwrap();
    let data = fs::read(input_path)?;

    if data.len() < NONCE_SIZE {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "File too short to contain nonce",
        ));
    }

    let (nonce_bytes, ciphertext) = data.split_at(NONCE_SIZE);
    let nonce = Nonce::from_slice(nonce_bytes);

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    fs::write(output_path, plaintext)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_roundtrip() {
        let key = [0x42u8; 32];
        let test_data = b"Secret data that needs protection";

        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        fs::write(input_file.path(), test_data).unwrap();

        encrypt_file(
            input_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap(),
            &key,
        )
        .unwrap();

        decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap(),
            &key,
        )
        .unwrap();

        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(decrypted_data, test_data);
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
    let mut content = Vec::new();
    file.read_to_end(&mut content)?;

    if content.len() < 32 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "File too short to contain key",
        ));
    }

    let (key_bytes, ciphertext) = content.split_at(32);
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let plaintext = b"Secret data to protect";
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        fs::write(input_file.path(), plaintext).unwrap();

        encrypt_file(input_file.path(), encrypted_file.path()).unwrap();
        decrypt_file(encrypted_file.path(), decrypted_file.path()).unwrap();

        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(decrypted_content, plaintext);
    }
}