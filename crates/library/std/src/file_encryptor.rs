use std::fs;
use std::io::{self, Read, Write};

const DEFAULT_KEY: u8 = 0x55;

fn xor_cipher(data: &mut [u8], key: u8) {
    for byte in data.iter_mut() {
        *byte ^= key;
    }
}

pub fn encrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    let key = key.unwrap_or(DEFAULT_KEY);
    let mut content = fs::read(input_path)?;
    xor_cipher(&mut content, key);
    fs::write(output_path, &content)
}

pub fn decrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    encrypt_file(input_path, output_path, key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_xor_cipher_symmetry() {
        let mut data = vec![0x00, 0xFF, 0xAA, 0x55];
        let original = data.clone();
        let key = 0x33;

        xor_cipher(&mut data, key);
        assert_ne!(data, original);

        xor_cipher(&mut data, key);
        assert_eq!(data, original);
    }

    #[test]
    fn test_file_encryption() -> io::Result<()> {
        let input_file = NamedTempFile::new()?;
        let output_file = NamedTempFile::new()?;
        let decrypted_file = NamedTempFile::new()?;

        let test_data = b"Secret message";
        fs::write(input_file.path(), test_data)?;

        encrypt_file(
            input_file.path().to_str().unwrap(),
            output_file.path().to_str().unwrap(),
            Some(0x42),
        )?;

        let encrypted = fs::read(output_file.path())?;
        assert_ne!(encrypted, test_data);

        decrypt_file(
            output_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap(),
            Some(0x42),
        )?;

        let decrypted = fs::read(decrypted_file.path())?;
        assert_eq!(decrypted, test_data);

        Ok(())
    }
}