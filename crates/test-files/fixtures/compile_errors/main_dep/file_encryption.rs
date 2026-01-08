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

    pub fn encrypt_file(&self, source_path: &str, dest_path: &str) -> io::Result<()> {
        self.process_file(source_path, dest_path)
    }

    pub fn decrypt_file(&self, source_path: &str, dest_path: &str) -> io::Result<()> {
        self.process_file(source_path, dest_path)
    }

    fn process_file(&self, source_path: &str, dest_path: &str) -> io::Result<()> {
        let mut source_file = fs::File::open(Path::new(source_path))?;
        let mut buffer = Vec::new();
        source_file.read_to_end(&mut buffer)?;

        let processed_data: Vec<u8> = buffer
            .iter()
            .enumerate()
            .map(|(i, &byte)| byte ^ self.key[i % self.key.len()])
            .collect();

        let mut dest_file = fs::File::create(Path::new(dest_path))?;
        dest_file.write_all(&processed_data)?;

        Ok(())
    }
}

pub fn generate_random_key(length: usize) -> Vec<u8> {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    (0..length).map(|_| rng.gen()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_xor_cipher_symmetry() {
        let cipher = XORCipher::new("secret_key");
        let test_data = b"Hello, XOR encryption!";
        
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
        
        assert_eq!(test_data.to_vec(), decrypted);
    }

    #[test]
    fn test_file_encryption() -> io::Result<()> {
        let cipher = XORCipher::new("test_key");
        let source_file = NamedTempFile::new()?;
        let encrypted_file = NamedTempFile::new()?;
        let decrypted_file = NamedTempFile::new()?;

        fs::write(source_file.path(), "Test file content")?;
        
        cipher.encrypt_file(
            source_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap(),
        )?;
        
        cipher.decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap(),
        )?;
        
        let original_content = fs::read_to_string(source_file.path())?;
        let decrypted_content = fs::read_to_string(decrypted_file.path())?;
        
        assert_eq!(original_content, decrypted_content);
        Ok(())
    }
}