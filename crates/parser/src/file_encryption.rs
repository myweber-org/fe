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

    pub fn encrypt_file(&self, source_path: &Path, dest_path: &Path) -> Result<(), String> {
        self.process_file(source_path, dest_path)
    }

    pub fn decrypt_file(&self, source_path: &Path, dest_path: &Path) -> Result<(), String> {
        self.process_file(source_path, dest_path)
    }

    fn process_file(&self, source_path: &Path, dest_path: &Path) -> Result<(), String> {
        let mut source_file = fs::File::open(source_path)
            .map_err(|e| format!("Failed to open source file: {}", e))?;

        let mut dest_file = fs::File::create(dest_path)
            .map_err(|e| format!("Failed to create destination file: {}", e))?;

        let mut buffer = [0u8; 4096];
        let mut key_index = 0;

        loop {
            let bytes_read = source_file.read(&mut buffer)
                .map_err(|e| format!("Failed to read from source file: {}", e))?;

            if bytes_read == 0 {
                break;
            }

            for i in 0..bytes_read {
                buffer[i] ^= self.key[key_index];
                key_index = (key_index + 1) % self.key.len();
            }

            dest_file.write_all(&buffer[..bytes_read])
                .map_err(|e| format!("Failed to write to destination file: {}", e))?;
        }

        dest_file.flush()
            .map_err(|e| format!("Failed to flush destination file: {}", e))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_xor_cipher_symmetry() {
        let cipher = XorCipher::new("secret_key");
        let original_data = b"Hello, this is a test message for XOR encryption!";
        
        let mut source_file = NamedTempFile::new().unwrap();
        source_file.write_all(original_data).unwrap();
        
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        cipher.encrypt_file(source_file.path(), encrypted_file.path()).unwrap();
        cipher.decrypt_file(encrypted_file.path(), decrypted_file.path()).unwrap();
        
        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(original_data.to_vec(), decrypted_data);
    }

    #[test]
    fn test_empty_key() {
        let cipher = XorCipher::new("");
        let data = b"Test data";
        
        let mut source_file = NamedTempFile::new().unwrap();
        source_file.write_all(data).unwrap();
        
        let dest_file = NamedTempFile::new().unwrap();
        
        let result = cipher.encrypt_file(source_file.path(), dest_file.path());
        assert!(result.is_ok());
        
        let processed_data = fs::read(dest_file.path()).unwrap();
        assert_eq!(data.to_vec(), processed_data);
    }
}