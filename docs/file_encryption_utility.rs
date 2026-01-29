use base64::{engine::general_purpose, Engine as _};
use std::fs;
use std::io::{self, Read, Write};

const XOR_KEY: u8 = 0xAA;

fn xor_cipher(data: &mut [u8]) {
    for byte in data.iter_mut() {
        *byte ^= XOR_KEY;
    }
}

pub fn encrypt_file(input_path: &str, output_path: &str) -> io::Result<()> {
    let mut file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    
    xor_cipher(&mut buffer);
    let encoded = general_purpose::STANDARD.encode(&buffer);
    
    let mut output = fs::File::create(output_path)?;
    output.write_all(encoded.as_bytes())?;
    
    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str) -> io::Result<()> {
    let mut file = fs::File::open(input_path)?;
    let mut encoded = String::new();
    file.read_to_string(&mut encoded)?;
    
    let decoded = general_purpose::STANDARD.decode(encoded.trim())?;
    let mut buffer = decoded;
    xor_cipher(&mut buffer);
    
    let mut output = fs::File::create(output_path)?;
    output.write_all(&buffer)?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_encryption_roundtrip() {
        let test_data = b"Secret data for encryption test!";
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), test_data).unwrap();
        
        encrypt_file(input_file.path().to_str().unwrap(), 
                    encrypted_file.path().to_str().unwrap()).unwrap();
        decrypt_file(encrypted_file.path().to_str().unwrap(),
                    decrypted_file.path().to_str().unwrap()).unwrap();
        
        let result = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(test_data, result.as_slice());
    }
}