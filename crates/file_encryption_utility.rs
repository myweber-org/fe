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

    pub fn encrypt_file(&self, source_path: &Path, dest_path: &Path) -> io::Result<()> {
        self.process_file(source_path, dest_path)
    }

    pub fn decrypt_file(&self, source_path: &Path, dest_path: &Path) -> io::Result<()> {
        self.process_file(source_path, dest_path)
    }

    fn process_file(&self, source_path: &Path, dest_path: &Path) -> io::Result<()> {
        let mut source_file = fs::File::open(source_path)?;
        let mut dest_file = fs::File::create(dest_path)?;

        let mut buffer = [0; 4096];
        let mut key_index = 0;

        loop {
            let bytes_read = source_file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }

            let mut processed_chunk = Vec::with_capacity(bytes_read);
            for i in 0..bytes_read {
                let key_byte = self.key[key_index % self.key.len()];
                processed_chunk.push(buffer[i] ^ key_byte);
                key_index += 1;
            }

            dest_file.write_all(&processed_chunk)?;
        }

        dest_file.flush()?;
        Ok(())
    }
}

pub fn validate_key(key: &str) -> Result<(), &'static str> {
    if key.is_empty() {
        return Err("Encryption key cannot be empty");
    }
    if key.len() < 8 {
        return Err("Encryption key must be at least 8 characters long");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_xor_cipher_symmetry() {
        let cipher = XORCipher::new("secure_key_123!");
        let test_data = b"Hello, this is a secret message!";

        let mut encrypted = Vec::new();
        for (i, &byte) in test_data.iter().enumerate() {
            let key_byte = cipher.key[i % cipher.key.len()];
            encrypted.push(byte ^ key_byte);
        }

        let mut decrypted = Vec::new();
        for (i, &byte) in encrypted.iter().enumerate() {
            let key_byte = cipher.key[i % cipher.key.len()];
            decrypted.push(byte ^ key_byte);
        }

        assert_eq!(test_data.to_vec(), decrypted);
    }

    #[test]
    fn test_file_encryption_decryption() -> io::Result<()> {
        let cipher = XORCipher::new("test_encryption_key_2024");
        
        let mut source_file = NamedTempFile::new()?;
        source_file.write_all(b"Test file content for encryption")?;
        
        let encrypted_file = NamedTempFile::new()?;
        let decrypted_file = NamedTempFile::new()?;

        cipher.encrypt_file(source_file.path(), encrypted_file.path())?;
        cipher.decrypt_file(encrypted_file.path(), decrypted_file.path())?;

        let original_content = fs::read(source_file.path())?;
        let decrypted_content = fs::read(decrypted_file.path())?;

        assert_eq!(original_content, decrypted_content);
        Ok(())
    }

    #[test]
    fn test_key_validation() {
        assert!(validate_key("").is_err());
        assert!(validate_key("short").is_err());
        assert!(validate_key("valid_key_long_enough").is_ok());
    }
}