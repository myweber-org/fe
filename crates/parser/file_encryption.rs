
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

pub fn xor_encrypt_file(input_path: &str, output_path: &str, key: &[u8]) -> io::Result<()> {
    let input_data = fs::read(input_path)?;
    let encrypted_data = xor_encrypt(&input_data, key);
    fs::write(output_path, encrypted_data)?;
    Ok(())
}

pub fn xor_decrypt_file(input_path: &str, output_path: &str, key: &[u8]) -> io::Result<()> {
    let input_data = fs::read(input_path)?;
    let decrypted_data = xor_decrypt(&input_data, key);
    fs::write(output_path, decrypted_data)?;
    Ok(())
}

fn xor_encrypt(data: &[u8], key: &[u8]) -> Vec<u8> {
    data.iter()
        .enumerate()
        .map(|(i, &byte)| byte ^ key[i % key.len()])
        .collect()
}

fn xor_decrypt(data: &[u8], key: &[u8]) -> Vec<u8> {
    xor_encrypt(data, key)
}

pub fn generate_random_key(length: usize) -> Vec<u8> {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    (0..length).map(|_| rng.gen()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_xor_encryption() {
        let data = b"Hello, World!";
        let key = b"secret";
        
        let encrypted = xor_encrypt(data, key);
        let decrypted = xor_decrypt(&encrypted, key);
        
        assert_eq!(data.to_vec(), decrypted);
    }

    #[test]
    fn test_file_encryption() -> io::Result<()> {
        let input_file = NamedTempFile::new()?;
        let output_file = NamedTempFile::new()?;
        let decrypted_file = NamedTempFile::new()?;
        
        let test_data = b"Test file content for encryption";
        fs::write(input_file.path(), test_data)?;
        
        let key = b"testkey123";
        
        xor_encrypt_file(
            input_file.path().to_str().unwrap(),
            output_file.path().to_str().unwrap(),
            key,
        )?;
        
        xor_decrypt_file(
            output_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap(),
            key,
        )?;
        
        let decrypted_content = fs::read(decrypted_file.path())?;
        assert_eq!(test_data.to_vec(), decrypted_content);
        
        Ok(())
    }
}