use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: u8 = 0xAA;

pub fn encrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    let key = key.unwrap_or(DEFAULT_KEY);
    let mut input_file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    input_file.read_to_end(&mut buffer)?;

    for byte in buffer.iter_mut() {
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
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let original_content = b"Secret data to protect";
        let key = 0xCC;

        let input_temp = NamedTempFile::new().unwrap();
        let encrypted_temp = NamedTempFile::new().unwrap();
        let decrypted_temp = NamedTempFile::new().unwrap();

        fs::write(input_temp.path(), original_content).unwrap();

        encrypt_file(
            input_temp.path().to_str().unwrap(),
            encrypted_temp.path().to_str().unwrap(),
            Some(key),
        ).unwrap();

        let encrypted_content = fs::read(encrypted_temp.path()).unwrap();
        assert_ne!(encrypted_content, original_content);

        decrypt_file(
            encrypted_temp.path().to_str().unwrap(),
            decrypted_temp.path().to_str().unwrap(),
            Some(key),
        ).unwrap();

        let decrypted_content = fs::read(decrypted_temp.path()).unwrap();
        assert_eq!(decrypted_content, original_content);
    }

    #[test]
    fn test_default_key() {
        let content = b"Test with default key";
        let input_temp = NamedTempFile::new().unwrap();
        let output_temp = NamedTempFile::new().unwrap();

        fs::write(input_temp.path(), content).unwrap();

        encrypt_file(
            input_temp.path().to_str().unwrap(),
            output_temp.path().to_str().unwrap(),
            None,
        ).unwrap();

        let encrypted = fs::read(output_temp.path()).unwrap();
        assert_ne!(encrypted, content);
    }
}