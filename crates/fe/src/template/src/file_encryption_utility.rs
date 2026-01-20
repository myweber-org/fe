use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: u8 = 0x55;

pub struct FileCipher {
    key: u8,
}

impl FileCipher {
    pub fn new(key: Option<u8>) -> Self {
        FileCipher {
            key: key.unwrap_or(DEFAULT_KEY),
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

        let mut buffer = [0u8; 4096];
        loop {
            let bytes_read = source_file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }

            let processed_chunk: Vec<u8> = buffer[..bytes_read]
                .iter()
                .map(|&byte| byte ^ self.key)
                .collect();

            dest_file.write_all(&processed_chunk)?;
        }

        dest_file.flush()?;
        Ok(())
    }

    pub fn encrypt_data(&self, data: &[u8]) -> Vec<u8> {
        data.iter().map(|&byte| byte ^ self.key).collect()
    }

    pub fn decrypt_data(&self, data: &[u8]) -> Vec<u8> {
        self.encrypt_data(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_symmetric_encryption() {
        let cipher = FileCipher::new(Some(0xAA));
        let original_data = b"Hello, World!";
        
        let encrypted = cipher.encrypt_data(original_data);
        let decrypted = cipher.decrypt_data(&encrypted);
        
        assert_eq!(original_data, decrypted.as_slice());
    }

    #[test]
    fn test_file_encryption() -> io::Result<()> {
        let cipher = FileCipher::new(None);
        
        let mut source_file = NamedTempFile::new()?;
        let test_data = b"Test file content for encryption";
        source_file.write_all(test_data)?;
        
        let dest_file = NamedTempFile::new()?;
        
        cipher.encrypt_file(source_file.path(), dest_file.path())?;
        
        let mut encrypted_content = Vec::new();
        fs::File::open(dest_file.path())?.read_to_end(&mut encrypted_content)?;
        
        assert_ne!(test_data, encrypted_content.as_slice());
        
        let mut decrypted_file = NamedTempFile::new()?;
        cipher.decrypt_file(dest_file.path(), decrypted_file.path())?;
        
        let mut decrypted_content = Vec::new();
        fs::File::open(decrypted_file.path())?.read_to_end(&mut decrypted_content)?;
        
        assert_eq!(test_data, decrypted_content.as_slice());
        Ok(())
    }
}