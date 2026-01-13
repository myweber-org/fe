
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

pub struct XorCipher {
    key: Vec<u8>,
    key_index: usize,
}

impl XorCipher {
    pub fn new(key: &str) -> Self {
        XorCipher {
            key: key.as_bytes().to_vec(),
            key_index: 0,
        }
    }

    fn next_key_byte(&mut self) -> u8 {
        let byte = self.key[self.key_index];
        self.key_index = (self.key_index + 1) % self.key.len();
        byte
    }

    pub fn process_bytes(&mut self, data: &[u8]) -> Vec<u8> {
        data.iter()
            .map(|&byte| byte ^ self.next_key_byte())
            .collect()
    }
}

pub fn encrypt_file(input_path: &Path, output_path: &Path, key: &str) -> io::Result<()> {
    let mut cipher = XorCipher::new(key);
    let mut input_file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    input_file.read_to_end(&mut buffer)?;

    let encrypted_data = cipher.process_bytes(&buffer);

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&encrypted_data)?;
    Ok(())
}

pub fn decrypt_file(input_path: &Path, output_path: &Path, key: &str) -> io::Result<()> {
    encrypt_file(input_path, output_path, key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_xor_cipher_symmetry() {
        let key = "secret_key";
        let original_data = b"Hello, World! This is a test message.";
        let mut cipher = XorCipher::new(key);

        let encrypted = cipher.process_bytes(original_data);
        let mut cipher2 = XorCipher::new(key);
        let decrypted = cipher2.process_bytes(&encrypted);

        assert_eq!(original_data.to_vec(), decrypted);
    }

    #[test]
    fn test_file_encryption() -> io::Result<()> {
        let key = "test_password";
        let original_content = b"Sample file content for encryption test.";

        let mut input_file = NamedTempFile::new()?;
        input_file.write_all(original_content)?;
        let output_file = NamedTempFile::new()?;

        encrypt_file(input_file.path(), output_file.path(), key)?;

        let mut cipher = XorCipher::new(key);
        let mut encrypted_content = Vec::new();
        fs::File::open(output_file.path())?.read_to_end(&mut encrypted_content)?;

        assert_ne!(original_content, encrypted_content.as_slice());

        let decrypted = cipher.process_bytes(&encrypted_content);
        assert_eq!(original_content, decrypted.as_slice());

        Ok(())
    }
}
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: u8 = 0x55;

pub fn xor_encrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    let encryption_key = key.unwrap_or(DEFAULT_KEY);
    
    let input_data = fs::read(input_path)?;
    let encrypted_data: Vec<u8> = input_data
        .iter()
        .map(|byte| byte ^ encryption_key)
        .collect();
    
    fs::write(output_path, encrypted_data)?;
    Ok(())
}

pub fn xor_decrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    xor_encrypt_file(input_path, output_path, key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let original_content = b"Hello, World! This is a test file.";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), original_content).unwrap();
        
        xor_encrypt_file(
            input_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap(),
            Some(0x42),
        ).unwrap();
        
        xor_decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap(),
            Some(0x42),
        ).unwrap();
        
        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(original_content.to_vec(), decrypted_content);
    }
}