
use std::fs;
use std::io::{self, Read, Write};

pub fn xor_cipher(data: &mut [u8], key: &[u8]) {
    for (i, byte) in data.iter_mut().enumerate() {
        *byte ^= key[i % key.len()];
    }
}

pub fn encrypt_file(input_path: &str, output_path: &str, key: &str) -> io::Result<()> {
    let mut file_content = fs::read(input_path)?;
    xor_cipher(&mut file_content, key.as_bytes());
    fs::write(output_path, &file_content)
}

pub fn decrypt_file(input_path: &str, output_path: &str, key: &str) -> io::Result<()> {
    encrypt_file(input_path, output_path, key)
}

#[cfg(test)]
mod tests {
    use super::*;
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
    fn test_file_encryption() -> io::Result<()> {
        let key = "test_key";
        let content = b"confidential data";

        let input_file = NamedTempFile::new()?;
        let output_file = NamedTempFile::new()?;
        let decrypted_file = NamedTempFile::new()?;

        fs::write(input_file.path(), content)?;

        encrypt_file(
            input_file.path().to_str().unwrap(),
            output_file.path().to_str().unwrap(),
            key,
        )?;

        let encrypted = fs::read(output_file.path())?;
        assert_ne!(encrypted, content);

        decrypt_file(
            output_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap(),
            key,
        )?;

        let decrypted = fs::read(decrypted_file.path())?;
        assert_eq!(decrypted, content);

        Ok(())
    }
}