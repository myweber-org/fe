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
        let mut input_file = fs::File::open(input_path)?;
        let mut output_file = fs::File::create(output_path)?;
        
        let mut buffer = [0u8; 4096];
        let key_len = self.key.len();
        let mut key_index = 0;
        
        loop {
            let bytes_read = input_file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            
            for i in 0..bytes_read {
                buffer[i] ^= self.key[key_index];
                key_index = (key_index + 1) % key_len;
            }
            
            output_file.write_all(&buffer[..bytes_read])?;
        }
        
        Ok(())
    }

    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path) -> io::Result<()> {
        self.encrypt_file(input_path, output_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_encryption_decryption() {
        let encryptor = FileEncryptor::new(None);
        let test_data = b"Hello, this is a test message for encryption!";
        
        let mut input_file = NamedTempFile::new().unwrap();
        input_file.write_all(test_data).unwrap();
        
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        encryptor.encrypt_file(input_file.path(), encrypted_file.path()).unwrap();
        encryptor.decrypt_file(encrypted_file.path(), decrypted_file.path()).unwrap();
        
        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(test_data.to_vec(), decrypted_data);
    }

    #[test]
    fn test_custom_key() {
        let custom_key = b"my-custom-key-123";
        let encryptor = FileEncryptor::new(Some(custom_key));
        let test_data = b"Sensitive information";
        
        let mut input_file = NamedTempFile::new().unwrap();
        input_file.write_all(test_data).unwrap();
        
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        encryptor.encrypt_file(input_file.path(), encrypted_file.path()).unwrap();
        
        let different_encryptor = FileEncryptor::new(Some(b"wrong-key"));
        different_encryptor.decrypt_file(encrypted_file.path(), decrypted_file.path()).unwrap();
        
        let wrong_decrypted = fs::read(decrypted_file.path()).unwrap();
        assert_ne!(test_data.to_vec(), wrong_decrypted);
        
        encryptor.decrypt_file(encrypted_file.path(), decrypted_file.path()).unwrap();
        let correct_decrypted = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(test_data.to_vec(), correct_decrypted);
    }
}