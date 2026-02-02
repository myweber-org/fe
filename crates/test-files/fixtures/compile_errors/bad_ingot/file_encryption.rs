use std::env;
use std::fs;
use std::io::{self, Read, Write};

fn xor_data(data: &[u8], key: &[u8]) -> Vec<u8> {
    data.iter()
        .enumerate()
        .map(|(i, &byte)| byte ^ key[i % key.len()])
        .collect()
}

fn process_file(input_path: &str, output_path: &str, key: &[u8]) -> io::Result<()> {
    let mut input_file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    input_file.read_to_end(&mut buffer)?;

    let processed_data = xor_data(&buffer, key);

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&processed_data)?;

    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() != 4 {
        eprintln!("Usage: {} <input_file> <output_file> <key>", args[0]);
        std::process::exit(1);
    }

    let input_path = &args[1];
    let output_path = &args[2];
    let key = args[3].as_bytes();

    if let Err(e) = process_file(input_path, output_path, key) {
        eprintln!("Error processing file: {}", e);
        std::process::exit(1);
    }

    println!("File processed successfully: {} -> {}", input_path, output_path);
}
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use std::fs;

pub struct FileEncryptor {
    cipher: Aes256Gcm,
}

impl FileEncryptor {
    pub fn new() -> Self {
        let key = Aes256Gcm::generate_key(&mut OsRng);
        let cipher = Aes256Gcm::new(&key);
        Self { cipher }
    }

    pub fn encrypt_file(&self, input_path: &str, output_path: &str) -> Result<(), String> {
        let data = fs::read(input_path)
            .map_err(|e| format!("Failed to read file: {}", e))?;
        
        let nonce = Nonce::from_slice(b"unique_nonce_12");
        let ciphertext = self.cipher.encrypt(nonce, data.as_ref())
            .map_err(|e| format!("Encryption failed: {}", e))?;
        
        fs::write(output_path, ciphertext)
            .map_err(|e| format!("Failed to write encrypted file: {}", e))?;
        
        Ok(())
    }

    pub fn decrypt_file(&self, input_path: &str, output_path: &str) -> Result<(), String> {
        let ciphertext = fs::read(input_path)
            .map_err(|e| format!("Failed to read encrypted file: {}", e))?;
        
        let nonce = Nonce::from_slice(b"unique_nonce_12");
        let plaintext = self.cipher.decrypt(nonce, ciphertext.as_ref())
            .map_err(|e| format!("Decryption failed: {}", e))?;
        
        fs::write(output_path, plaintext)
            .map_err(|e| format!("Failed to write decrypted file: {}", e))?;
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let encryptor = FileEncryptor::new();
        let test_data = b"Secret data for encryption test";
        
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
        assert_eq!(decrypted_data, test_data);
    }
}