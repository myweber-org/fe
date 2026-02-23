
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use std::fs;
use std::io::{self, Read, Write};

const NONCE_SIZE: usize = 12;

pub fn encrypt_file(input_path: &str, output_path: &str) -> io::Result<()> {
    let key = Key::<Aes256Gcm>::generate(&mut OsRng);
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(&[0u8; NONCE_SIZE]);

    let mut file = fs::File::open(input_path)?;
    let mut plaintext = Vec::new();
    file.read_to_end(&mut plaintext)?;

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_ref())
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&ciphertext)?;

    println!("Encryption successful. Key: {}", hex::encode(key));
    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str, key_hex: &str) -> io::Result<()> {
    let key_bytes = hex::decode(key_hex)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(&[0u8; NONCE_SIZE]);

    let mut file = fs::File::open(input_path)?;
    let mut ciphertext = Vec::new();
    file.read_to_end(&mut ciphertext)?;

    let plaintext = cipher
        .decrypt(nonce, ciphertext.as_ref())
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&plaintext)?;

    println!("Decryption successful.");
    Ok(())
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

        let key = hex::encode(Key::<Aes256Gcm>::generate(&mut OsRng));
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&hex::decode(&key).unwrap()));

        encrypt_file(
            input_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap(),
        ).unwrap();

        decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap(),
            &key,
        ).unwrap();

        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(plaintext.to_vec(), decrypted_data);
    }
}use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use std::fs;
use std::io::{self, Write};

const NONCE_SIZE: usize = 12;

pub fn encrypt_file(input_path: &str, output_path: &str, password: &str) -> io::Result<()> {
    let data = fs::read(input_path)?;
    
    let key = derive_key(password);
    let cipher = Aes256Gcm::new(&key);
    
    let mut nonce_bytes = [0u8; NONCE_SIZE];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    
    let ciphertext = cipher
        .encrypt(nonce, data.as_ref())
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    
    let mut output_data = Vec::with_capacity(NONCE_SIZE + ciphertext.len());
    output_data.extend_from_slice(&nonce_bytes);
    output_data.extend_from_slice(&ciphertext);
    
    fs::write(output_path, output_data)
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
    
    fs::write(output_path, plaintext)
}

fn derive_key(password: &str) -> Key<Aes256Gcm> {
    let mut key_bytes = [0u8; 32];
    let password_bytes = password.as_bytes();
    
    for (i, &byte) in password_bytes.iter().enumerate() {
        key_bytes[i % 32] ^= byte.wrapping_mul(31);
    }
    
    *Key::<Aes256Gcm>::from_slice(&key_bytes)
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
        
        encrypt_file(
            input_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap(),
            password,
        ).unwrap();
        
        decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap(),
            password,
        ).unwrap();
        
        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(original_data.as_slice(), decrypted_data.as_slice());
    }
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

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() != 3 {
        eprintln!("Usage: {} <input_file> <output_file>", args[0]);
        std::process::exit(1);
    }
    
    let input_path = Path::new(&args[1]);
    let output_path = Path::new(&args[2]);
    
    if !input_path.exists() {
        eprintln!("Error: Input file does not exist");
        std::process::exit(1);
    }
    
    process_file(input_path, output_path, DEFAULT_KEY)?;
    
    println!("File processed successfully with XOR key 0x{:02X}", DEFAULT_KEY);
    Ok(())
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
        let key = 0x55;
        
        xor_cipher(&mut data, key);
        assert_ne!(data, original);
        
        xor_cipher(&mut data, key);
        assert_eq!(data, original);
    }
    
    #[test]
    fn test_file_encryption() -> io::Result<()> {
        let test_data = b"Hello, World!";
        let input_file = NamedTempFile::new()?;
        let output_file = NamedTempFile::new()?;
        
        fs::write(input_file.path(), test_data)?;
        
        process_file(input_file.path(), output_file.path(), DEFAULT_KEY)?;
        
        let encrypted = fs::read(output_file.path())?;
        assert_ne!(encrypted, test_data);
        
        let mut decrypted = encrypted.clone();
        xor_cipher(&mut decrypted, DEFAULT_KEY);
        assert_eq!(decrypted, test_data);
        
        Ok(())
    }
}