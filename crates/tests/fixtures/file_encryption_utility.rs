use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: &[u8] = b"secret-encryption-key-2024";

pub struct FileEncryptor {
    key: Vec<u8>,
}

impl FileEncryptor {
    pub fn new(key: Option<&[u8]>) -> Self {
        let key = match key {
            Some(k) => k.to_vec(),
            None => DEFAULT_KEY.to_vec(),
        };
        FileEncryptor { key }
    }

    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> io::Result<()> {
        self.process_file(input_path, output_path, true)
    }

    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path) -> io::Result<()> {
        self.process_file(input_path, output_path, false)
    }

    fn process_file(&self, input_path: &Path, output_path: &Path, is_encrypt: bool) -> io::Result<()> {
        let mut input_file = fs::File::open(input_path)?;
        let mut output_file = fs::File::create(output_path)?;

        let mut buffer = [0u8; 4096];
        let mut key_index = 0;

        loop {
            let bytes_read = input_file.read(&mut buffer)?;
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

            output_file.write_all(&processed_chunk)?;
        }

        output_file.flush()?;
        Ok(())
    }

    pub fn encrypt_string(&self, text: &str) -> Vec<u8> {
        self.xor_transform(text.as_bytes())
    }

    pub fn decrypt_string(&self, data: &[u8]) -> String {
        let decrypted = self.xor_transform(data);
        String::from_utf8_lossy(&decrypted).to_string()
    }

    fn xor_transform(&self, data: &[u8]) -> Vec<u8> {
        data.iter()
            .enumerate()
            .map(|(i, &byte)| byte ^ self.key[i % self.key.len()])
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_string_encryption_decryption() {
        let encryptor = FileEncryptor::new(None);
        let original = "Hello, World! This is a test message.";
        
        let encrypted = encryptor.encrypt_string(original);
        let decrypted = encryptor.decrypt_string(&encrypted);
        
        assert_eq!(original, decrypted);
    }

    #[test]
    fn test_file_encryption_decryption() {
        let encryptor = FileEncryptor::new(Some(b"test-key"));
        let test_data = b"Sample file content for encryption testing.";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), test_data).unwrap();
        
        encryptor.encrypt_file(input_file.path(), encrypted_file.path()).unwrap();
        encryptor.decrypt_file(encrypted_file.path(), decrypted_file.path()).unwrap();
        
        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(test_data.to_vec(), decrypted_content);
    }
}