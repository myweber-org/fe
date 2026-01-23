use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

pub fn xor_cipher(data: &mut [u8], key: &[u8]) {
    for (i, byte) in data.iter_mut().enumerate() {
        *byte ^= key[i % key.len()];
    }
}

pub fn process_file(input_path: &Path, output_path: &Path, key: &[u8]) -> io::Result<()> {
    let mut file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    
    xor_cipher(&mut buffer, key);
    
    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&buffer)?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_xor_cipher_symmetry() {
        let key = b"secret";
        let original = b"Hello, World!";
        let mut data = original.to_vec();
        
        xor_cipher(&mut data, key);
        assert_ne!(data.as_slice(), original);
        
        xor_cipher(&mut data, key);
        assert_eq!(data.as_slice(), original);
    }

    #[test]
    fn test_file_encryption() -> io::Result<()> {
        let key = b"test_key";
        let content = b"Sample file content for encryption test";
        
        let mut input_file = NamedTempFile::new()?;
        input_file.write_all(content)?;
        
        let output_file = NamedTempFile::new()?;
        
        process_file(input_file.path(), output_file.path(), key)?;
        
        let mut encrypted = Vec::new();
        fs::File::open(output_file.path())?.read_to_end(&mut encrypted)?;
        assert_ne!(encrypted.as_slice(), content);
        
        xor_cipher(&mut encrypted, key);
        assert_eq!(encrypted.as_slice(), content);
        
        Ok(())
    }
}