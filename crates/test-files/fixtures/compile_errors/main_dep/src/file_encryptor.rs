
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
        let test_data = b"Hello, World!";
        let key = b"secret";
        let input_path = "test_input.txt";
        let encrypted_path = "test_encrypted.txt";
        let decrypted_path = "test_decrypted.txt";

        fs::write(input_path, test_data).unwrap();

        xor_encrypt_file(input_path, encrypted_path, key).unwrap();
        xor_decrypt_file(encrypted_path, decrypted_path, key).unwrap();

        let decrypted_data = fs::read(decrypted_path).unwrap();
        assert_eq!(decrypted_data, test_data);

        fs::remove_file(input_path).unwrap_or_default();
        fs::remove_file(encrypted_path).unwrap_or_default();
        fs::remove_file(decrypted_path).unwrap_or_default();
    }
}