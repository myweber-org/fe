
use std::fs;
use std::io::{self, Read, Write};

const DEFAULT_KEY: u8 = 0x55;

pub fn encrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    let key = key.unwrap_or(DEFAULT_KEY);
    
    let mut input_file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    input_file.read_to_end(&mut buffer)?;
    
    for byte in &mut buffer {
        *byte ^= key;
    }
    
    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&buffer)?;
    
    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    encrypt_file(input_path, output_path, key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    
    #[test]
    fn test_encryption_decryption() {
        let test_data = b"Hello, World!";
        let test_file = "test_input.txt";
        let encrypted_file = "test_encrypted.bin";
        let decrypted_file = "test_decrypted.txt";
        
        fs::write(test_file, test_data).unwrap();
        
        encrypt_file(test_file, encrypted_file, Some(0x42)).unwrap();
        decrypt_file(encrypted_file, decrypted_file, Some(0x42)).unwrap();
        
        let decrypted_data = fs::read(decrypted_file).unwrap();
        assert_eq!(decrypted_data, test_data);
        
        fs::remove_file(test_file).unwrap();
        fs::remove_file(encrypted_file).unwrap();
        fs::remove_file(decrypted_file).unwrap();
    }
}