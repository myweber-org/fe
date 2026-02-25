use std::fs;
use std::io::{Read, Write};

const DEFAULT_KEY: u8 = 0xAA;

pub fn encrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> Result<(), String> {
    let key = key.unwrap_or(DEFAULT_KEY);
    
    let mut input_file = fs::File::open(input_path)
        .map_err(|e| format!("Failed to open input file: {}", e))?;
    
    let mut buffer = Vec::new();
    input_file.read_to_end(&mut buffer)
        .map_err(|e| format!("Failed to read input file: {}", e))?;
    
    let encrypted_data: Vec<u8> = buffer.iter()
        .map(|byte| byte ^ key)
        .collect();
    
    let mut output_file = fs::File::create(output_path)
        .map_err(|e| format!("Failed to create output file: {}", e))?;
    
    output_file.write_all(&encrypted_data)
        .map_err(|e| format!("Failed to write output file: {}", e))?;
    
    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> Result<(), String> {
    encrypt_file(input_path, output_path, key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let original_content = b"Hello, Rust encryption!";
        
        let input_temp = NamedTempFile::new().unwrap();
        let encrypted_temp = NamedTempFile::new().unwrap();
        let decrypted_temp = NamedTempFile::new().unwrap();
        
        fs::write(input_temp.path(), original_content).unwrap();
        
        encrypt_file(
            input_temp.path().to_str().unwrap(),
            encrypted_temp.path().to_str().unwrap(),
            Some(0x55)
        ).unwrap();
        
        let encrypted_content = fs::read(encrypted_temp.path()).unwrap();
        assert_ne!(encrypted_content, original_content);
        
        decrypt_file(
            encrypted_temp.path().to_str().unwrap(),
            decrypted_temp.path().to_str().unwrap(),
            Some(0x55)
        ).unwrap();
        
        let decrypted_content = fs::read(decrypted_temp.path()).unwrap();
        assert_eq!(decrypted_content, original_content);
    }
}