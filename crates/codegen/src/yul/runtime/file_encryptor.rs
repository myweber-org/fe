
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

pub struct FileEncryptor {
    key: Vec<u8>,
}

impl FileEncryptor {
    pub fn new(key: &str) -> Self {
        FileEncryptor {
            key: key.as_bytes().to_vec(),
        }
    }

    pub fn encrypt_file(&self, source_path: &str, dest_path: &str) -> io::Result<()> {
        self.process_file(source_path, dest_path, true)
    }

    pub fn decrypt_file(&self, source_path: &str, dest_path: &str) -> io::Result<()> {
        self.process_file(source_path, dest_path, false)
    }

    fn process_file(&self, source_path: &str, dest_path: &str, is_encrypt: bool) -> io::Result<()> {
        let source = Path::new(source_path);
        let dest = Path::new(dest_path);

        if !source.exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Source file not found: {}", source_path),
            ));
        }

        let mut source_file = fs::File::open(source)?;
        let mut dest_file = fs::File::create(dest)?;

        let mut buffer = [0; 4096];
        let mut key_index = 0;

        loop {
            let bytes_read = source_file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }

            let mut processed_buffer = buffer[..bytes_read].to_vec();
            
            for byte in processed_buffer.iter_mut() {
                if !self.key.is_empty() {
                    *byte ^= self.key[key_index];
                    key_index = (key_index + 1) % self.key.len();
                }
            }

            dest_file.write_all(&processed_buffer)?;
        }

        dest_file.flush()?;
        
        if is_encrypt {
            println!("File encrypted successfully: {} -> {}", source_path, dest_path);
        } else {
            println!("File decrypted successfully: {} -> {}", source_path, dest_path);
        }

        Ok(())
    }

    pub fn encrypt_string(&self, text: &str) -> Vec<u8> {
        self.process_bytes(text.as_bytes())
    }

    pub fn decrypt_string(&self, data: &[u8]) -> String {
        let processed = self.process_bytes(data);
        String::from_utf8_lossy(&processed).to_string()
    }

    fn process_bytes(&self, data: &[u8]) -> Vec<u8> {
        let mut result = Vec::with_capacity(data.len());
        let mut key_index = 0;

        for &byte in data {
            let processed_byte = if !self.key.is_empty() {
                byte ^ self.key[key_index]
            } else {
                byte
            };
            result.push(processed_byte);
            key_index = (key_index + 1) % self.key.len();
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encrypt_decrypt_string() {
        let encryptor = FileEncryptor::new("secret_key");
        let original = "Hello, World!";
        
        let encrypted = encryptor.encrypt_string(original);
        let decrypted = encryptor.decrypt_string(&encrypted);
        
        assert_ne!(encrypted, original.as_bytes());
        assert_eq!(decrypted, original);
    }

    #[test]
    fn test_file_encryption() {
        let encryptor = FileEncryptor::new("test_key");
        let original_content = b"This is a test file content.";
        
        let source_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(source_file.path(), original_content).unwrap();
        
        encryptor.encrypt_file(
            source_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap()
        ).unwrap();
        
        encryptor.decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap()
        ).unwrap();
        
        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(decrypted_content, original_content);
    }
}