
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
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: u8 = 0x55;

pub fn encrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    let encryption_key = key.unwrap_or(DEFAULT_KEY);
    let data = fs::read(input_path)?;
    
    let encrypted_data: Vec<u8> = data.iter()
        .map(|byte| byte ^ encryption_key)
        .collect();
    
    fs::write(output_path, encrypted_data)?;
    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    encrypt_file(input_path, output_path, key)
}

pub fn process_file_interactive() -> io::Result<()> {
    println!("Enter input file path:");
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input_path = input.trim();
    
    println!("Enter output file path:");
    let mut output = String::new();
    io::stdin().read_line(&mut output)?;
    let output_path = output.trim();
    
    println!("Enter encryption key (0-255, empty for default):");
    let mut key_input = String::new();
    io::stdin().read_line(&mut key_input)?;
    
    let key = key_input.trim()
        .parse::<u8>()
        .ok();
    
    println!("Encrypt (e) or Decrypt (d)?");
    let mut mode = String::new();
    io::stdin().read_line(&mut mode)?;
    
    match mode.trim().to_lowercase().as_str() {
        "e" => encrypt_file(input_path, output_path, key),
        "d" => decrypt_file(input_path, output_path, key),
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Invalid mode selected"
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_encryption_decryption() {
        let original_data = b"Hello, World!";
        let temp_input = NamedTempFile::new().unwrap();
        let temp_encrypted = NamedTempFile::new().unwrap();
        let temp_decrypted = NamedTempFile::new().unwrap();
        
        fs::write(temp_input.path(), original_data).unwrap();
        
        encrypt_file(
            temp_input.path().to_str().unwrap(),
            temp_encrypted.path().to_str().unwrap(),
            Some(0x42)
        ).unwrap();
        
        decrypt_file(
            temp_encrypted.path().to_str().unwrap(),
            temp_decrypted.path().to_str().unwrap(),
            Some(0x42)
        ).unwrap();
        
        let decrypted_data = fs::read(temp_decrypted.path()).unwrap();
        assert_eq!(original_data.to_vec(), decrypted_data);
    }
}