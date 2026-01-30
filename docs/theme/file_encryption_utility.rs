use std::fs::{self, File};
use std::io::{Read, Write};
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

    pub fn encrypt_file(&self, source_path: &Path, dest_path: &Path) -> Result<(), String> {
        self.process_file(source_path, dest_path, true)
    }

    pub fn decrypt_file(&self, source_path: &Path, dest_path: &Path) -> Result<(), String> {
        self.process_file(source_path, dest_path, false)
    }

    fn process_file(&self, source_path: &Path, dest_path: &Path, is_encrypt: bool) -> Result<(), String> {
        if !source_path.exists() {
            return Err(format!("Source file does not exist: {:?}", source_path));
        }

        let mut source_file = File::open(source_path)
            .map_err(|e| format!("Failed to open source file: {}", e))?;
        
        let mut dest_file = File::create(dest_path)
            .map_err(|e| format!("Failed to create destination file: {}", e))?;

        let mut buffer = Vec::new();
        source_file.read_to_end(&mut buffer)
            .map_err(|e| format!("Failed to read source file: {}", e))?;

        let processed_data = self.xor_transform(&buffer);

        dest_file.write_all(&processed_data)
            .map_err(|e| format!("Failed to write destination file: {}", e))?;

        if is_encrypt {
            println!("File encrypted successfully: {:?} -> {:?}", source_path, dest_path);
        } else {
            println!("File decrypted successfully: {:?} -> {:?}", source_path, dest_path);
        }

        Ok(())
    }

    fn xor_transform(&self, data: &[u8]) -> Vec<u8> {
        let key_len = self.key.len();
        if key_len == 0 {
            return data.to_vec();
        }

        data.iter()
            .enumerate()
            .map(|(i, &byte)| byte ^ self.key[i % key_len])
            .collect()
    }
}

pub fn calculate_file_hash(path: &Path) -> Result<String, String> {
    let mut file = File::open(path)
        .map_err(|e| format!("Failed to open file for hashing: {}", e))?;
    
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)
        .map_err(|e| format!("Failed to read file for hashing: {}", e))?;

    let hash = sha256::digest(&buffer);
    Ok(hash)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_xor_cipher_symmetry() {
        let cipher = XORCipher::new("test_key_123");
        let original_data = b"Hello, this is a secret message!";
        
        let encrypted = cipher.xor_transform(original_data);
        let decrypted = cipher.xor_transform(&encrypted);
        
        assert_eq!(original_data.to_vec(), decrypted);
    }

    #[test]
    fn test_file_encryption_decryption() {
        let cipher = XORCipher::new("secure_password");
        
        let mut source_file = NamedTempFile::new().unwrap();
        let test_content = b"Confidential data that needs protection";
        source_file.write_all(test_content).unwrap();
        
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        let source_path = source_file.path();
        let encrypted_path = encrypted_file.path();
        let decrypted_path = decrypted_file.path();
        
        assert!(cipher.encrypt_file(source_path, encrypted_path).is_ok());
        assert!(cipher.decrypt_file(encrypted_path, decrypted_path).is_ok());
        
        let decrypted_content = fs::read(decrypted_path).unwrap();
        assert_eq!(test_content.to_vec(), decrypted_content);
    }

    #[test]
    fn test_empty_key() {
        let cipher = XORCipher::new("");
        let data = b"Some data";
        let transformed = cipher.xor_transform(data);
        assert_eq!(data.to_vec(), transformed);
    }
}