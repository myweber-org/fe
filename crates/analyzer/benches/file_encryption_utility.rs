
use std::fs;
use std::io::{Read, Write};
use std::path::Path;

pub struct XorCipher {
    key: Vec<u8>,
}

impl XorCipher {
    pub fn new(key: &str) -> Self {
        XorCipher {
            key: key.as_bytes().to_vec(),
        }
    }

    pub fn encrypt_file(&self, source_path: &str, dest_path: &str) -> Result<(), String> {
        self.process_file(source_path, dest_path, true)
    }

    pub fn decrypt_file(&self, source_path: &str, dest_path: &str) -> Result<(), String> {
        self.process_file(source_path, dest_path, false)
    }

    fn process_file(&self, source_path: &str, dest_path: &str, is_encrypt: bool) -> Result<(), String> {
        let source = Path::new(source_path);
        let dest = Path::new(dest_path);

        if !source.exists() {
            return Err(format!("Source file does not exist: {}", source_path));
        }

        let mut source_file = fs::File::open(source)
            .map_err(|e| format!("Failed to open source file: {}", e))?;

        let mut dest_file = fs::File::create(dest)
            .map_err(|e| format!("Failed to create destination file: {}", e))?;

        let mut buffer = [0u8; 4096];
        let mut key_index = 0;

        loop {
            let bytes_read = source_file.read(&mut buffer)
                .map_err(|e| format!("Failed to read from source file: {}", e))?;

            if bytes_read == 0 {
                break;
            }

            let processed_chunk: Vec<u8> = buffer[..bytes_read]
                .iter()
                .map(|&byte| {
                    let result = byte ^ self.key[key_index];
                    key_index = (key_index + 1) % self.key.len();
                    result
                })
                .collect();

            dest_file.write_all(&processed_chunk)
                .map_err(|e| format!("Failed to write to destination file: {}", e))?;
        }

        dest_file.flush()
            .map_err(|e| format!("Failed to flush destination file: {}", e))?;

        if is_encrypt {
            println!("File encrypted successfully: {} -> {}", source_path, dest_path);
        } else {
            println!("File decrypted successfully: {} -> {}", source_path, dest_path);
        }

        Ok(())
    }

    pub fn encrypt_data(&self, data: &[u8]) -> Vec<u8> {
        let mut result = Vec::with_capacity(data.len());
        let mut key_index = 0;

        for &byte in data {
            result.push(byte ^ self.key[key_index]);
            key_index = (key_index + 1) % self.key.len();
        }

        result
    }

    pub fn decrypt_data(&self, data: &[u8]) -> Vec<u8> {
        self.encrypt_data(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_xor_cipher_symmetry() {
        let cipher = XorCipher::new("secret_key");
        let original_data = b"Hello, World! This is a test message.";
        
        let encrypted = cipher.encrypt_data(original_data);
        let decrypted = cipher.decrypt_data(&encrypted);
        
        assert_eq!(original_data, decrypted.as_slice());
    }

    #[test]
    fn test_file_encryption_decryption() {
        let cipher = XorCipher::new("test_password_123");
        
        let original_content = b"Sample file content for encryption testing.";
        
        let source_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(source_file.path(), original_content).unwrap();
        
        cipher.encrypt_file(
            source_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap()
        ).unwrap();
        
        cipher.decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap()
        ).unwrap();
        
        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(original_content, decrypted_content.as_slice());
    }
}