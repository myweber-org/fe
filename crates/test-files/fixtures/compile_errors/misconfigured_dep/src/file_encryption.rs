use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

pub fn xor_encrypt_file(input_path: &str, output_path: &str, key: &[u8]) -> io::Result<()> {
    let input_data = fs::read(input_path)?;
    let encrypted_data: Vec<u8> = input_data
        .iter()
        .enumerate()
        .map(|(i, &byte)| byte ^ key[i % key.len()])
        .collect();
    
    fs::write(output_path, encrypted_data)?;
    Ok(())
}

pub fn xor_decrypt_file(input_path: &str, output_path: &str, key: &[u8]) -> io::Result<()> {
    xor_encrypt_file(input_path, output_path, key)
}

pub fn process_file() -> io::Result<()> {
    let key = b"secret_key";
    let original = "test_data.txt";
    let encrypted = "encrypted.bin";
    let decrypted = "decrypted.txt";

    if !Path::new(original).exists() {
        let mut file = fs::File::create(original)?;
        file.write_all(b"Sample data for encryption test.")?;
    }

    xor_encrypt_file(original, encrypted, key)?;
    println!("File encrypted successfully.");

    xor_decrypt_file(encrypted, decrypted, key)?;
    println!("File decrypted successfully.");

    let restored = fs::read_to_string(decrypted)?;
    println!("Decrypted content: {}", restored);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_xor_roundtrip() {
        let key = b"test_key";
        let test_data = b"Hello, XOR encryption!";
        let test_file = "test_roundtrip.txt";
        let enc_file = "test_enc.bin";
        let dec_file = "test_dec.txt";

        fs::write(test_file, test_data).unwrap();
        xor_encrypt_file(test_file, enc_file, key).unwrap();
        xor_decrypt_file(enc_file, dec_file, key).unwrap();

        let result = fs::read(dec_file).unwrap();
        assert_eq!(test_data.to_vec(), result);

        fs::remove_file(test_file).ok();
        fs::remove_file(enc_file).ok();
        fs::remove_file(dec_file).ok();
    }
}