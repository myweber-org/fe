use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const BUFFER_SIZE: usize = 8192;

pub fn xor_encrypt_file(input_path: &str, output_path: &str, key: &[u8]) -> io::Result<()> {
    let input_path = Path::new(input_path);
    let output_path = Path::new(output_path);
    
    if !input_path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Input file not found: {}", input_path.display())
        ));
    }
    
    let mut input_file = fs::File::open(input_path)?;
    let mut output_file = fs::File::create(output_path)?;
    
    let mut buffer = [0u8; BUFFER_SIZE];
    let key_len = key.len();
    let mut key_index = 0;
    
    loop {
        let bytes_read = input_file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        
        for i in 0..bytes_read {
            buffer[i] ^= key[key_index];
            key_index = (key_index + 1) % key_len;
        }
        
        output_file.write_all(&buffer[..bytes_read])?;
    }
    
    output_file.flush()?;
    Ok(())
}

pub fn xor_decrypt_file(input_path: &str, output_path: &str, key: &[u8]) -> io::Result<()> {
    xor_encrypt_file(input_path, output_path, key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;
    
    #[test]
    fn test_xor_encryption_decryption() {
        let original_content = b"Hello, this is a secret message!";
        let key = b"mysecretkey123";
        
        let mut input_file = NamedTempFile::new().unwrap();
        input_file.write_all(original_content).unwrap();
        
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        let input_path = input_file.path().to_str().unwrap();
        let encrypted_path = encrypted_file.path().to_str().unwrap();
        let decrypted_path = decrypted_file.path().to_str().unwrap();
        
        xor_encrypt_file(input_path, encrypted_path, key).unwrap();
        xor_decrypt_file(encrypted_path, decrypted_path, key).unwrap();
        
        let decrypted_content = fs::read(decrypted_path).unwrap();
        assert_eq!(original_content, decrypted_content.as_slice());
    }
}