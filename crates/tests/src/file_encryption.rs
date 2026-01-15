
use std::fs;
use std::io::{Read, Write};
use std::path::Path;

pub struct XorCipher {
    key: Vec<u8>,
    key_position: usize,
}

impl XorCipher {
    pub fn new(key: &str) -> Self {
        XorCipher {
            key: key.as_bytes().to_vec(),
            key_position: 0,
        }
    }

    fn next_key_byte(&mut self) -> u8 {
        let byte = self.key[self.key_position];
        self.key_position = (self.key_position + 1) % self.key.len();
        byte
    }

    pub fn encrypt(&mut self, data: &[u8]) -> Vec<u8> {
        self.key_position = 0;
        data.iter()
            .map(|&byte| byte ^ self.next_key_byte())
            .collect()
    }

    pub fn decrypt(&mut self, data: &[u8]) -> Vec<u8> {
        self.encrypt(data)
    }
}

pub fn process_file(input_path: &Path, output_path: &Path, key: &str, encrypt: bool) -> Result<(), String> {
    let mut cipher = XorCipher::new(key);
    
    let mut input_file = fs::File::open(input_path)
        .map_err(|e| format!("Failed to open input file: {}", e))?;
    
    let mut buffer = Vec::new();
    input_file.read_to_end(&mut buffer)
        .map_err(|e| format!("Failed to read input file: {}", e))?;
    
    let processed_data = if encrypt {
        cipher.encrypt(&buffer)
    } else {
        cipher.decrypt(&buffer)
    };
    
    let mut output_file = fs::File::create(output_path)
        .map_err(|e| format!("Failed to create output file: {}", e))?;
    
    output_file.write_all(&processed_data)
        .map_err(|e| format!("Failed to write output file: {}", e))?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_xor_cipher_symmetry() {
        let key = "secret_key_123";
        let mut cipher = XorCipher::new(key);
        
        let original_data = b"Hello, World! This is a test message.";
        let encrypted = cipher.encrypt(original_data);
        
        let mut cipher2 = XorCipher::new(key);
        let decrypted = cipher2.decrypt(&encrypted);
        
        assert_eq!(original_data.to_vec(), decrypted);
    }

    #[test]
    fn test_file_encryption() {
        let key = "test_key";
        let original_content = b"Sample file content for encryption test.";
        
        let input_file = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), original_content).unwrap();
        
        process_file(input_file.path(), output_file.path(), key, true)
            .expect("Encryption should succeed");
        
        let encrypted_content = fs::read(output_file.path()).unwrap();
        assert_ne!(original_content, encrypted_content.as_slice());
        
        let decrypted_file = NamedTempFile::new().unwrap();
        process_file(output_file.path(), decrypted_file.path(), key, false)
            .expect("Decryption should succeed");
        
        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(original_content, decrypted_content.as_slice());
    }
}