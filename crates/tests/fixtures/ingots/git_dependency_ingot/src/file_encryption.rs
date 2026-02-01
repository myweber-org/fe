
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const BUFFER_SIZE: usize = 8192;

pub fn encrypt_file(input_path: &str, output_path: &str, key: &[u8]) -> io::Result<()> {
    process_file(input_path, output_path, key)
}

pub fn decrypt_file(input_path: &str, output_path: &str, key: &[u8]) -> io::Result<()> {
    process_file(input_path, output_path, key)
}

fn process_file(input_path: &str, output_path: &str, key: &[u8]) -> io::Result<()> {
    let input_path = Path::new(input_path);
    let output_path = Path::new(output_path);

    if !input_path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Input file not found: {}", input_path.display()),
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let original_content = b"Hello, this is a test message for encryption!";
        let key = b"secret_key";

        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        fs::write(input_file.path(), original_content).unwrap();

        encrypt_file(
            input_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap(),
            key,
        )
        .unwrap();

        let encrypted_content = fs::read(encrypted_file.path()).unwrap();
        assert_ne!(encrypted_content, original_content);

        decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap(),
            key,
        )
        .unwrap();

        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(decrypted_content, original_content);
    }

    #[test]
    fn test_empty_file() {
        let key = b"test_key";
        let input_file = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();

        encrypt_file(
            input_file.path().to_str().unwrap(),
            output_file.path().to_str().unwrap(),
            key,
        )
        .unwrap();

        let output_content = fs::read(output_file.path()).unwrap();
        assert!(output_content.is_empty());
    }
}