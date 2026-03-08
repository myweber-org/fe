
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: u8 = 0x55;

fn xor_encrypt_decrypt(data: &mut [u8], key: u8) {
    for byte in data.iter_mut() {
        *byte ^= key;
    }
}

fn process_file(input_path: &Path, output_path: &Path, key: u8) -> io::Result<()> {
    let mut file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    xor_encrypt_decrypt(&mut buffer, key);

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&buffer)?;

    Ok(())
}

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <input_file> <output_file>", args[0]);
        std::process::exit(1);
    }

    let input_path = Path::new(&args[1]);
    let output_path = Path::new(&args[2]);

    if !input_path.exists() {
        eprintln!("Error: Input file does not exist");
        std::process::exit(1);
    }

    process_file(input_path, output_path, DEFAULT_KEY)?;
    println!("File processed successfully with XOR key 0x{:02X}", DEFAULT_KEY);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_xor_encrypt_decrypt() {
        let mut data = vec![0x00, 0xFF, 0x55, 0xAA];
        let original = data.clone();
        let key = 0x55;

        xor_encrypt_decrypt(&mut data, key);
        assert_ne!(data, original);

        xor_encrypt_decrypt(&mut data, key);
        assert_eq!(data, original);
    }

    #[test]
    fn test_file_processing() -> io::Result<()> {
        let mut input_file = NamedTempFile::new()?;
        let test_data = b"Hello, XOR encryption!";
        input_file.write_all(test_data)?;

        let output_file = NamedTempFile::new()?;
        let input_path = input_file.path();
        let output_path = output_file.path();

        process_file(input_path, output_path, DEFAULT_KEY)?;

        let mut encrypted_data = Vec::new();
        fs::File::open(output_path)?.read_to_end(&mut encrypted_data)?;
        assert_ne!(encrypted_data, test_data);

        let mut temp_file = NamedTempFile::new()?;
        temp_file.write_all(&encrypted_data)?;
        process_file(temp_file.path(), output_path, DEFAULT_KEY)?;

        let mut decrypted_data = Vec::new();
        fs::File::open(output_path)?.read_to_end(&mut decrypted_data)?;
        assert_eq!(decrypted_data, test_data);

        Ok(())
    }
}