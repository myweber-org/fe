
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

pub fn xor_encrypt_file(input_path: &str, output_path: &str, key: &[u8]) -> io::Result<()> {
    let input_data = fs::read(input_path)?;
    let encrypted_data = xor_transform(&input_data, key);
    fs::write(output_path, encrypted_data)?;
    Ok(())
}

pub fn xor_decrypt_file(input_path: &str, output_path: &str, key: &[u8]) -> io::Result<()> {
    xor_encrypt_file(input_path, output_path, key)
}

fn xor_transform(data: &[u8], key: &[u8]) -> Vec<u8> {
    let mut result = Vec::with_capacity(data.len());
    for (i, &byte) in data.iter().enumerate() {
        result.push(byte ^ key[i % key.len()]);
    }
    result
}

pub fn process_stream<R: Read, W: Write>(mut reader: R, mut writer: W, key: &[u8]) -> io::Result<()> {
    let mut buffer = [0; 4096];
    let mut key_index = 0;
    
    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        
        for i in 0..bytes_read {
            buffer[i] ^= key[key_index];
            key_index = (key_index + 1) % key.len();
        }
        
        writer.write_all(&buffer[..bytes_read])?;
    }
    
    writer.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    
    #[test]
    fn test_xor_transform() {
        let data = b"Hello World";
        let key = b"secret";
        let encrypted = xor_transform(data, key);
        let decrypted = xor_transform(&encrypted, key);
        assert_eq!(data.to_vec(), decrypted);
    }
    
    #[test]
    fn test_stream_processing() {
        let input = b"Test data stream";
        let key = b"key123";
        let mut reader = Cursor::new(input);
        let mut writer = Vec::new();
        
        process_stream(&mut reader, &mut writer, key).unwrap();
        let encrypted = writer;
        
        let mut reader2 = Cursor::new(&encrypted);
        let mut writer2 = Vec::new();
        process_stream(&mut reader2, &mut writer2, key).unwrap();
        
        assert_eq!(input.to_vec(), writer2);
    }
}