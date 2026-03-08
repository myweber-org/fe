use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: u8 = 0xAA;

pub fn encrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    let encryption_key = key.unwrap_or(DEFAULT_KEY);
    let mut input_file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    input_file.read_to_end(&mut buffer)?;

    for byte in &mut buffer {
        *byte ^= encryption_key;
    }

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&buffer)?;
    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    encrypt_file(input_path, output_path, key)
}

pub fn process_directory(dir_path: &str, key: Option<u8>, encrypt: bool) -> io::Result<()> {
    let dir_entries = fs::read_dir(dir_path)?;
    let operation = if encrypt { "encrypted" } else { "decrypted" };
    let key_value = key.unwrap_or(DEFAULT_KEY);

    for entry in dir_entries {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_file() {
            let input_str = path.to_str().unwrap();
            let output_name = format!("{}_{}", operation, path.file_name().unwrap().to_str().unwrap());
            let output_path = path.with_file_name(output_name);

            if encrypt {
                encrypt_file(input_str, output_path.to_str().unwrap(), Some(key_value))?;
            } else {
                decrypt_file(input_str, output_path.to_str().unwrap(), Some(key_value))?;
            }
            
            println!("Processed: {}", path.display());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let original_data = b"Test data for encryption";
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(original_data).unwrap();
        
        let input_path = temp_file.path().to_str().unwrap();
        let encrypted_path = "test_encrypted.bin";
        let decrypted_path = "test_decrypted.bin";
        
        encrypt_file(input_path, encrypted_path, Some(0x55)).unwrap();
        decrypt_file(encrypted_path, decrypted_path, Some(0x55)).unwrap();
        
        let decrypted_data = fs::read(decrypted_path).unwrap();
        assert_eq!(original_data, decrypted_data.as_slice());
        
        fs::remove_file(encrypted_path).unwrap();
        fs::remove_file(decrypted_path).unwrap();
    }
}