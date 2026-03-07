
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;

pub struct XorCipher {
    key: Vec<u8>,
}

impl XorCipher {
    pub fn new(key: &str) -> Self {
        Self {
            key: key.as_bytes().to_vec(),
        }
    }

    pub fn encrypt_file(&self, source_path: &Path, dest_path: &Path) -> Result<(), String> {
        let mut source_file = File::open(source_path)
            .map_err(|e| format!("Failed to open source file: {}", e))?;
        
        let mut content = Vec::new();
        source_file.read_to_end(&mut content)
            .map_err(|e| format!("Failed to read source file: {}", e))?;

        let encrypted = self.xor_transform(&content);
        
        let mut dest_file = File::create(dest_path)
            .map_err(|e| format!("Failed to create destination file: {}", e))?;
        
        dest_file.write_all(&encrypted)
            .map_err(|e| format!("Failed to write encrypted data: {}", e))?;

        Ok(())
    }

    pub fn decrypt_file(&self, source_path: &Path, dest_path: &Path) -> Result<(), String> {
        self.encrypt_file(source_path, dest_path)
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

pub fn encrypt_directory(dir_path: &Path, key: &str, extension: &str) -> Result<(), String> {
    if !dir_path.is_dir() {
        return Err("Provided path is not a directory".to_string());
    }

    let cipher = XorCipher::new(key);
    
    for entry in fs::read_dir(dir_path)
        .map_err(|e| format!("Failed to read directory: {}", e))? 
    {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let path = entry.path();
        
        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext == extension {
                    let encrypted_path = path.with_extension(format!("{}.enc", extension));
                    cipher.encrypt_file(&path, &encrypted_path)?;
                    println!("Encrypted: {:?} -> {:?}", path, encrypted_path);
                }
            }
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_xor_cipher_symmetry() {
        let cipher = XorCipher::new("secret_key");
        let original = b"Hello, World!";
        
        let encrypted = cipher.xor_transform(original);
        let decrypted = cipher.xor_transform(&encrypted);
        
        assert_eq!(original.to_vec(), decrypted);
    }

    #[test]
    fn test_empty_key() {
        let cipher = XorCipher::new("");
        let data = b"Test data";
        
        let result = cipher.xor_transform(data);
        assert_eq!(data.to_vec(), result);
    }
}