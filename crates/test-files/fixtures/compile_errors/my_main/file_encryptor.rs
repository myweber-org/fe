
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

pub fn generate_key_from_string(s: &str) -> Vec<u8> {
    s.bytes().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_xor_cipher_symmetry() {
        let key = b"secret";
        let mut data = b"hello world".to_vec();
        let original = data.clone();

        xor_cipher(&mut data, key);
        assert_ne!(data, original);

        xor_cipher(&mut data, key);
        assert_eq!(data, original);
    }

    #[test]
    fn test_file_processing() -> io::Result<()> {
        let key = b"testkey";
        let mut input_file = NamedTempFile::new()?;
        write!(input_file, "Test file content")?;

        let output_file = NamedTempFile::new()?;
        
        process_file(input_file.path(), output_file.path(), key)?;

        let mut encrypted_content = Vec::new();
        fs::File::open(output_file.path())?.read_to_end(&mut encrypted_content)?;
        
        assert_ne!(encrypted_content, b"Test file content");

        let mut double_processed = NamedTempFile::new()?;
        process_file(output_file.path(), double_processed.path(), key)?;

        let mut final_content = String::new();
        fs::File::open(double_processed.path())?.read_to_string(&mut final_content)?;

        assert_eq!(final_content, "Test file content");

        Ok(())
    }
}