use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: u8 = 0x55;

pub fn encrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    let encryption_key = key.unwrap_or(DEFAULT_KEY);
    let mut input_file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    input_file.read_to_end(&mut buffer)?;

    for byte in buffer.iter_mut() {
        *byte ^= encryption_key;
    }

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&buffer)?;
    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    encrypt_file(input_path, output_path, key)
}

pub fn process_directory(dir_path: &str, operation: &str, key: Option<u8>) -> io::Result<()> {
    let dir_entries = fs::read_dir(dir_path)?;
    
    for entry in dir_entries {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_file() {
            let input_str = path.to_str().unwrap();
            let output_str = format!("{}.processed", input_str);
            
            match operation {
                "encrypt" => encrypt_file(input_str, &output_str, key)?,
                "decrypt" => decrypt_file(input_str, &output_str, key)?,
                _ => return Err(io::Error::new(io::ErrorKind::InvalidInput, "Invalid operation")),
            }
            
            println!("Processed: {}", input_str);
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
        let test_data = b"Hello, World!";
        let key = Some(0xAA);
        
        let mut input_file = NamedTempFile::new().unwrap();
        input_file.write_all(test_data).unwrap();
        
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        let input_path = input_file.path().to_str().unwrap();
        let encrypted_path = encrypted_file.path().to_str().unwrap();
        let decrypted_path = decrypted_file.path().to_str().unwrap();
        
        encrypt_file(input_path, encrypted_path, key).unwrap();
        decrypt_file(encrypted_path, decrypted_path, key).unwrap();
        
        let mut decrypted_data = Vec::new();
        fs::File::open(decrypted_path)
            .unwrap()
            .read_to_end(&mut decrypted_data)
            .unwrap();
            
        assert_eq!(test_data, decrypted_data.as_slice());
    }
}