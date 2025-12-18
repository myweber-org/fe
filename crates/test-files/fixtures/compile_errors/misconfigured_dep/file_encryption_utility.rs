use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: u8 = 0xAA;

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

pub fn process_stream<R: Read, W: Write>(mut reader: R, mut writer: W, key: u8) -> io::Result<()> {
    let mut buffer = [0; 1024];
    
    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        
        for byte in buffer[..bytes_read].iter_mut() {
            *byte ^= key;
        }
        
        writer.write_all(&buffer[..bytes_read])?;
    }
    
    writer.flush()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    
    #[test]
    fn test_xor_encryption() {
        let original = b"Hello, World!";
        let key = 0xCC;
        
        let encrypted: Vec<u8> = original.iter().map(|b| b ^ key).collect();
        let decrypted: Vec<u8> = encrypted.iter().map(|b| b ^ key).collect();
        
        assert_eq!(original.to_vec(), decrypted);
    }
    
    #[test]
    fn test_stream_processing() {
        let input = b"Test data stream";
        let key = 0x55;
        
        let mut reader = Cursor::new(input.to_vec());
        let mut writer = Cursor::new(Vec::new());
        
        process_stream(&mut reader, &mut writer, key).unwrap();
        
        let encrypted = writer.into_inner();
        let mut reader2 = Cursor::new(encrypted.clone());
        let mut writer2 = Cursor::new(Vec::new());
        
        process_stream(&mut reader2, &mut writer2, key).unwrap();
        
        assert_eq!(input.to_vec(), writer2.into_inner());
    }
}