use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use std::fs;

pub fn encrypt_file(input_path: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let key = Aes256Gcm::generate_key(&mut OsRng);
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(b"unique_nonce_12");

    let plaintext = fs::read(input_path)?;
    let ciphertext = cipher.encrypt(nonce, plaintext.as_ref())
        .map_err(|e| format!("Encryption failed: {}", e))?;

    let mut output_data = key.to_vec();
    output_data.extend_from_slice(&ciphertext);
    fs::write(output_path, output_data)?;

    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let data = fs::read(input_path)?;
    if data.len() < 32 {
        return Err("Invalid encrypted file format".into());
    }

    let (key_bytes, ciphertext) = data.split_at(32);
    let key = key_bytes.try_into()
        .map_err(|_| "Invalid key length")?;
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(b"unique_nonce_12");

    let plaintext = cipher.decrypt(nonce, ciphertext)
        .map_err(|e| format!("Decryption failed: {}", e))?;

    fs::write(output_path, plaintext)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_roundtrip() {
        let mut plain_file = NamedTempFile::new().unwrap();
        writeln!(plain_file, "Sensitive data: {}", "TestContent123").unwrap();

        let mut encrypted_file = NamedTempFile::new().unwrap();
        let mut decrypted_file = NamedTempFile::new().unwrap();

        encrypt_file(plain_file.path().to_str().unwrap(),
                    encrypted_file.path().to_str().unwrap()).unwrap();
        decrypt_file(encrypted_file.path().to_str().unwrap(),
                    decrypted_file.path().to_str().unwrap()).unwrap();

        let original = fs::read_to_string(plain_file.path()).unwrap();
        let decrypted = fs::read_to_string(decrypted_file.path()).unwrap();
        assert_eq!(original, decrypted);
    }
}