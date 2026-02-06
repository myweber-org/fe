use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: u8 = 0x55;

pub fn encrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    let encryption_key = key.unwrap_or(DEFAULT_KEY);
    let mut input_file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    input_file.read_to_end(&mut buffer)?;

    let encrypted_data: Vec<u8> = buffer.iter().map(|byte| byte ^ encryption_key).collect();

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&encrypted_data)?;

    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    encrypt_file(input_path, output_path, key)
}

pub fn process_directory(dir_path: &str, operation: fn(&str, &str, Option<u8>) -> io::Result<()>, key: Option<u8>) -> io::Result<()> {
    let path = Path::new(dir_path);
    if !path.is_dir() {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "Provided path is not a directory"));
    }

    for entry in fs::read_dir(dir_path)? {
        let entry = entry?;
        let file_path = entry.path();
        if file_path.is_file() {
            let input_str = file_path.to_str().unwrap();
            let output_str = format!("{}.processed", input_str);
            operation(input_str, &output_str, key)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let original_data = b"Hello, World!";
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(original_data).unwrap();

        let input_path = temp_file.path().to_str().unwrap();
        let encrypted_path = format!("{}.enc", input_path);
        let decrypted_path = format!("{}.dec", input_path);

        encrypt_file(input_path, &encrypted_path, Some(0x42)).unwrap();
        decrypt_file(&encrypted_path, &decrypted_path, Some(0x42)).unwrap();

        let decrypted_data = fs::read(decrypted_path).unwrap();
        assert_eq!(original_data.to_vec(), decrypted_data);
    }
}use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: u8 = 0xAA;

pub fn encrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    let encryption_key = key.unwrap_or(DEFAULT_KEY);
    
    let mut input_file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    input_file.read_to_end(&mut buffer)?;
    
    let encrypted_data: Vec<u8> = buffer
        .iter()
        .map(|byte| byte ^ encryption_key)
        .collect();
    
    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&encrypted_data)?;
    
    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    encrypt_file(input_path, output_path, key)
}

pub fn encrypt_string(text: &str, key: Option<u8>) -> Vec<u8> {
    let encryption_key = key.unwrap_or(DEFAULT_KEY);
    text.bytes()
        .map(|byte| byte ^ encryption_key)
        .collect()
}

pub fn decrypt_string(data: &[u8], key: Option<u8>) -> String {
    let encryption_key = key.unwrap_or(DEFAULT_KEY);
    data.iter()
        .map(|byte| (byte ^ encryption_key) as char)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_string_encryption() {
        let original = "Hello, World!";
        let encrypted = encrypt_string(original, Some(0x55));
        let decrypted = decrypt_string(&encrypted, Some(0x55));
        
        assert_eq!(original, decrypted);
    }
    
    #[test]
    fn test_file_encryption() -> io::Result<()> {
        let original_content = b"Test file content";
        
        let input_file = NamedTempFile::new()?;
        let output_file = NamedTempFile::new()?;
        
        fs::write(input_file.path(), original_content)?;
        
        encrypt_file(
            input_file.path().to_str().unwrap(),
            output_file.path().to_str().unwrap(),
            Some(0xCC),
        )?;
        
        let encrypted_content = fs::read(output_file.path())?;
        assert_ne!(original_content, encrypted_content.as_slice());
        
        decrypt_file(
            output_file.path().to_str().unwrap(),
            input_file.path().to_str().unwrap(),
            Some(0xCC),
        )?;
        
        let decrypted_content = fs::read(input_file.path())?;
        assert_eq!(original_content, decrypted_content.as_slice());
        
        Ok(())
    }
}