
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: u8 = 0x55;

pub fn encrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    let encryption_key = key.unwrap_or(DEFAULT_KEY);
    let data = fs::read(input_path)?;
    
    let encrypted_data: Vec<u8> = data
        .iter()
        .map(|byte| byte ^ encryption_key)
        .collect();
    
    fs::write(output_path, encrypted_data)
}

pub fn decrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    encrypt_file(input_path, output_path, key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_encryption_decryption() {
        let original_text = b"Hello, World!";
        let key = Some(0x42);
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), original_text).unwrap();
        
        encrypt_file(
            input_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap(),
            key
        ).unwrap();
        
        decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap(),
            key
        ).unwrap();
        
        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(decrypted_data, original_text);
    }
}
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: u8 = 0x55;

pub fn encrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    let encryption_key = key.unwrap_or(DEFAULT_KEY);
    
    let mut input_file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    input_file.read_to_end(&mut buffer)?;
    
    let encrypted_data: Vec<u8> = buffer.iter()
        .map(|byte| byte ^ encryption_key)
        .collect();
    
    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&encrypted_data)?;
    
    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    encrypt_file(input_path, output_path, key)
}

pub fn process_file(input_path: &str, output_path: &str, key: Option<u8>, encrypt: bool) -> io::Result<()> {
    if !Path::new(input_path).exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Input file '{}' not found", input_path)
        ));
    }
    
    if encrypt {
        encrypt_file(input_path, output_path, key)
    } else {
        decrypt_file(input_path, output_path, key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_encryption_decryption() {
        let original_text = b"Hello, Rust! This is a test message.";
        
        let input_temp = NamedTempFile::new().unwrap();
        let encrypted_temp = NamedTempFile::new().unwrap();
        let decrypted_temp = NamedTempFile::new().unwrap();
        
        fs::write(input_temp.path(), original_text).unwrap();
        
        encrypt_file(
            input_temp.path().to_str().unwrap(),
            encrypted_temp.path().to_str().unwrap(),
            Some(0x42)
        ).unwrap();
        
        decrypt_file(
            encrypted_temp.path().to_str().unwrap(),
            decrypted_temp.path().to_str().unwrap(),
            Some(0x42)
        ).unwrap();
        
        let decrypted_data = fs::read(decrypted_temp.path()).unwrap();
        assert_eq!(original_text.to_vec(), decrypted_data);
    }
    
    #[test]
    fn test_default_key() {
        let test_data = b"Test data for default key";
        
        let input_temp = NamedTempFile::new().unwrap();
        let output_temp = NamedTempFile::new().unwrap();
        
        fs::write(input_temp.path(), test_data).unwrap();
        
        encrypt_file(
            input_temp.path().to_str().unwrap(),
            output_temp.path().to_str().unwrap(),
            None
        ).unwrap();
        
        let encrypted = fs::read(output_temp.path()).unwrap();
        let decrypted: Vec<u8> = encrypted.iter()
            .map(|byte| byte ^ DEFAULT_KEY)
            .collect();
        
        assert_eq!(test_data.to_vec(), decrypted);
    }
}