
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

pub struct XORCipher {
    key: Vec<u8>,
}

impl XORCipher {
    pub fn new(key: &str) -> Self {
        XORCipher {
            key: key.as_bytes().to_vec(),
        }
    }

    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> io::Result<()> {
        self.process_file(input_path, output_path)
    }

    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path) -> io::Result<()> {
        self.process_file(input_path, output_path)
    }

    fn process_file(&self, input_path: &Path, output_path: &Path) -> io::Result<()> {
        let mut input_file = fs::File::open(input_path)?;
        let mut buffer = Vec::new();
        input_file.read_to_end(&mut buffer)?;

        let processed_data: Vec<u8> = buffer
            .iter()
            .enumerate()
            .map(|(i, &byte)| byte ^ self.key[i % self.key.len()])
            .collect();

        let mut output_file = fs::File::create(output_path)?;
        output_file.write_all(&processed_data)?;

        Ok(())
    }
}

pub fn validate_key(key: &str) -> Result<(), String> {
    if key.is_empty() {
        return Err("Encryption key cannot be empty".to_string());
    }
    if key.len() < 8 {
        return Err("Encryption key should be at least 8 characters".to_string());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_xor_cipher() {
        let cipher = XORCipher::new("secret_key_123");
        let test_data = b"Hello, World!";
        
        let encrypted: Vec<u8> = test_data
            .iter()
            .enumerate()
            .map(|(i, &byte)| byte ^ cipher.key[i % cipher.key.len()])
            .collect();
        
        let decrypted: Vec<u8> = encrypted
            .iter()
            .enumerate()
            .map(|(i, &byte)| byte ^ cipher.key[i % cipher.key.len()])
            .collect();
        
        assert_eq!(decrypted, test_data);
    }

    #[test]
    fn test_file_encryption() {
        let cipher = XORCipher::new("test_key_456");
        
        let mut input_file = NamedTempFile::new().unwrap();
        input_file.write_all(b"Test file content").unwrap();
        
        let output_file = NamedTempFile::new().unwrap();
        
        cipher.encrypt_file(input_file.path(), output_file.path()).unwrap();
        
        let decrypted_file = NamedTempFile::new().unwrap();
        cipher.decrypt_file(output_file.path(), decrypted_file.path()).unwrap();
        
        let mut content = String::new();
        fs::File::open(decrypted_file.path())
            .unwrap()
            .read_to_string(&mut content)
            .unwrap();
        
        assert_eq!(content, "Test file content");
    }

    #[test]
    fn test_key_validation() {
        assert!(validate_key("").is_err());
        assert!(validate_key("short").is_err());
        assert!(validate_key("valid_key_123").is_ok());
    }
}