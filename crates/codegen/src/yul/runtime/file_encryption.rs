
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: u8 = 0xAA;

fn xor_cipher(data: &mut [u8], key: u8) {
    for byte in data {
        *byte ^= key;
    }
}

fn process_file(input_path: &str, output_path: &str, key: u8) -> io::Result<()> {
    let mut file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    xor_cipher(&mut buffer, key);

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&buffer)?;

    Ok(())
}

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() != 4 {
        eprintln!("Usage: {} <encrypt|decrypt> <input_file> <output_file>", args[0]);
        std::process::exit(1);
    }

    let mode = &args[1];
    let input_file = &args[2];
    let output_file = &args[3];

    if !Path::new(input_file).exists() {
        eprintln!("Error: Input file '{}' does not exist", input_file);
        std::process::exit(1);
    }

    let key = match mode.as_str() {
        "encrypt" | "decrypt" => DEFAULT_KEY,
        _ => {
            eprintln!("Error: Mode must be 'encrypt' or 'decrypt'");
            std::process::exit(1);
        }
    };

    process_file(input_file, output_file, key)?;
    
    println!("{} completed successfully: {} -> {}", 
             mode, input_file, output_file);
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;

    #[test]
    fn test_xor_cipher_symmetry() {
        let mut data = vec![0x00, 0xFF, 0x55, 0xAA];
        let original = data.clone();
        let key = 0xCC;

        xor_cipher(&mut data, key);
        assert_ne!(data, original);

        xor_cipher(&mut data, key);
        assert_eq!(data, original);
    }

    #[test]
    fn test_file_encryption_decryption() -> io::Result<()> {
        let test_data = b"Hello, this is a secret message!";
        
        let input_temp = NamedTempFile::new()?;
        let encrypted_temp = NamedTempFile::new()?;
        let decrypted_temp = NamedTempFile::new()?;

        fs::write(input_temp.path(), test_data)?;

        process_file(
            input_temp.path().to_str().unwrap(),
            encrypted_temp.path().to_str().unwrap(),
            DEFAULT_KEY
        )?;

        let encrypted_content = fs::read(encrypted_temp.path())?;
        assert_ne!(encrypted_content, test_data);

        process_file(
            encrypted_temp.path().to_str().unwrap(),
            decrypted_temp.path().to_str().unwrap(),
            DEFAULT_KEY
        )?;

        let decrypted_content = fs::read(decrypted_temp.path())?;
        assert_eq!(decrypted_content, test_data);

        Ok(())
    }
}