
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: u8 = 0x55;

fn xor_crypt(data: &mut [u8], key: u8) {
    for byte in data.iter_mut() {
        *byte ^= key;
    }
}

pub fn encrypt_file(input_path: &Path, output_path: &Path, key: Option<u8>) -> io::Result<()> {
    let key = key.unwrap_or(DEFAULT_KEY);
    let mut content = fs::read(input_path)?;
    xor_crypt(&mut content, key);
    fs::write(output_path, content)
}

pub fn decrypt_file(input_path: &Path, output_path: &Path, key: Option<u8>) -> io::Result<()> {
    encrypt_file(input_path, output_path, key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::NamedTempFile;

    #[test]
    fn test_xor_crypt() {
        let mut data = vec![0x00, 0xFF, 0xAA, 0x55];
        xor_crypt(&mut data, 0xAA);
        assert_eq!(data, vec![0xAA, 0x55, 0x00, 0xFF]);
        xor_crypt(&mut data, 0xAA);
        assert_eq!(data, vec![0x00, 0xFF, 0xAA, 0x55]);
    }

    #[test]
    fn test_file_encryption() -> io::Result<()> {
        let original = b"Secret data";
        let input_temp = NamedTempFile::new()?;
        fs::write(input_temp.path(), original)?;

        let output_temp = NamedTempFile::new()?;
        encrypt_file(input_temp.path(), output_temp.path(), Some(0x42))?;

        let encrypted = fs::read(output_temp.path())?;
        assert_ne!(encrypted, original);

        let decrypt_temp = NamedTempFile::new()?;
        decrypt_file(output_temp.path(), decrypt_temp.path(), Some(0x42))?;
        let decrypted = fs::read(decrypt_temp.path())?;
        assert_eq!(decrypted, original);

        Ok(())
    }
}