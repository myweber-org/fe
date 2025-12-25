use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const BUFFER_SIZE: usize = 8192;

pub fn xor_cipher(data: &mut [u8], key: &[u8]) {
    let key_len = key.len();
    for (i, byte) in data.iter_mut().enumerate() {
        *byte ^= key[i % key_len];
    }
}

pub fn process_file(input_path: &Path, output_path: &Path, key: &[u8]) -> io::Result<()> {
    let mut input_file = fs::File::open(input_path)?;
    let mut output_file = fs::File::create(output_path)?;
    
    let mut buffer = [0u8; BUFFER_SIZE];
    
    loop {
        let bytes_read = input_file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        
        let data_slice = &mut buffer[..bytes_read];
        xor_cipher(data_slice, key);
        output_file.write_all(data_slice)?;
    }
    
    output_file.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_xor_cipher_symmetry() {
        let key = b"secret_key";
        let original = b"Hello, World!";
        let mut data = original.to_vec();
        
        xor_cipher(&mut data, key);
        assert_ne!(data.as_slice(), original);
        
        xor_cipher(&mut data, key);
        assert_eq!(data.as_slice(), original);
    }
    
    #[test]
    fn test_file_processing() -> io::Result<()> {
        let key = b"test_key_123";
        let test_data = b"Sample content for encryption test";
        
        let input_file = NamedTempFile::new()?;
        let output_file = NamedTempFile::new()?;
        
        fs::write(input_file.path(), test_data)?;
        
        process_file(input_file.path(), output_file.path(), key)?;
        
        let encrypted = fs::read(output_file.path())?;
        assert_ne!(encrypted.as_slice(), test_data);
        
        let mut double_encrypted = encrypted.clone();
        xor_cipher(&mut double_encrypted, key);
        assert_eq!(double_encrypted.as_slice(), test_data);
        
        Ok(())
    }
}