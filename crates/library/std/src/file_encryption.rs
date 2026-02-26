
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

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
    fs::write(output_path, content)?;
    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    encrypt_file(input_path, output_path, key)
}

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 4 {
        eprintln!("Usage: {} <encrypt|decrypt> <input> <output> [key]", args[0]);
        std::process::exit(1);
    }

    let operation = &args[1];
    let input = &args[2];
    let output = &args[3];
    let key = if args.len() > 4 {
        Some(args[4].parse::<u8>().unwrap_or(DEFAULT_KEY))
    } else {
        None
    };

    if !Path::new(input).exists() {
        return Err(io::Error::new(io::ErrorKind::NotFound, "Input file not found"));
    }

    match operation.as_str() {
        "encrypt" => encrypt_file(input, output, key),
        "decrypt" => decrypt_file(input, output, key),
        _ => {
            eprintln!("Invalid operation. Use 'encrypt' or 'decrypt'");
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_xor_cipher() {
        let mut data = vec![0x00, 0xFF, 0xAA, 0x55];
        let original = data.clone();
        xor_cipher(&mut data, 0xAA);
        xor_cipher(&mut data, 0xAA);
        assert_eq!(data, original);
    }

    #[test]
    fn test_file_encryption() -> io::Result<()> {
        let content = b"Secret data to encrypt";
        let input_file = NamedTempFile::new()?;
        let output_file = NamedTempFile::new()?;
        let decrypted_file = NamedTempFile::new()?;

        fs::write(input_file.path(), content)?;

        encrypt_file(input_file.path().to_str().unwrap(), output_file.path().to_str().unwrap(), Some(0x42))?;
        decrypt_file(output_file.path().to_str().unwrap(), decrypted_file.path().to_str().unwrap(), Some(0x42))?;

        let decrypted_content = fs::read(decrypted_file.path())?;
        assert_eq!(decrypted_content, content);

        Ok(())
    }
}