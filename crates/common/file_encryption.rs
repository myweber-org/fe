
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

    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), String> {
        self.process_file(input_path, output_path, true)
    }

    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), String> {
        self.process_file(input_path, output_path, false)
    }

    fn process_file(&self, input_path: &Path, output_path: &Path, _is_encrypt: bool) -> Result<(), String> {
        if !input_path.exists() {
            return Err(format!("Input file does not exist: {}", input_path.display()));
        }

        let mut input_file = fs::File::open(input_path)
            .map_err(|e| format!("Failed to open input file: {}", e))?;

        let mut buffer = Vec::new();
        input_file.read_to_end(&mut buffer)
            .map_err(|e| format!("Failed to read input file: {}", e))?;

        let processed_data = self.xor_transform(&buffer);

        let mut output_file = fs::File::create(output_path)
            .map_err(|e| format!("Failed to create output file: {}", e))?;

        output_file.write_all(&processed_data)
            .map_err(|e| format!("Failed to write output file: {}", e))?;

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;

    #[test]
    fn test_xor_cipher() {
        let cipher = XorCipher::new("secret_key");
        let test_data = b"Hello, World!";
        
        let encrypted = cipher.xor_transform(test_data);
        assert_ne!(encrypted, test_data);
        
        let cipher2 = XorCipher::new("secret_key");
        let decrypted = cipher2.xor_transform(&encrypted);
        assert_eq!(decrypted, test_data);
    }

    #[test]
    fn test_file_encryption() {
        let cipher = XorCipher::new("test_key");
        
        let input_file = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), "Test file content").unwrap();
        
        cipher.encrypt_file(input_file.path(), output_file.path()).unwrap();
        cipher.decrypt_file(output_file.path(), decrypted_file.path()).unwrap();
        
        let original = fs::read_to_string(input_file.path()).unwrap();
        let decrypted = fs::read_to_string(decrypted_file.path()).unwrap();
        
        assert_eq!(original, decrypted);
    }
}