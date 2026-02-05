
use std::fs;
use std::io::{self, Read, Write};

pub fn xor_encrypt_file(input_path: &str, output_path: &str, key: &[u8]) -> io::Result<()> {
    let mut input_file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    input_file.read_to_end(&mut buffer)?;

    let encrypted_data: Vec<u8> = buffer
        .iter()
        .enumerate()
        .map(|(i, &byte)| byte ^ key[i % key.len()])
        .collect();

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&encrypted_data)?;

    Ok(())
}

pub fn xor_decrypt_file(input_path: &str, output_path: &str, key: &[u8]) -> io::Result<()> {
    xor_encrypt_file(input_path, output_path, key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_xor_encryption() {
        let test_data = b"Hello, Rust!";
        let key = b"secret";
        let temp_input = "test_input.txt";
        let temp_encrypted = "test_encrypted.txt";
        let temp_decrypted = "test_decrypted.txt";

        fs::write(temp_input, test_data).unwrap();

        xor_encrypt_file(temp_input, temp_encrypted, key).unwrap();
        xor_decrypt_file(temp_encrypted, temp_decrypted, key).unwrap();

        let decrypted_data = fs::read(temp_decrypted).unwrap();
        assert_eq!(decrypted_data, test_data);

        fs::remove_file(temp_input).unwrap_or_default();
        fs::remove_file(temp_encrypted).unwrap_or_default();
        fs::remove_file(temp_decrypted).unwrap_or_default();
    }
}