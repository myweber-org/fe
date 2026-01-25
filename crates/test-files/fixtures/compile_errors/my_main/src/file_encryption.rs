use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use std::fs;

pub fn encrypt_file(input_path: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let key = Aes256Gcm::generate_key(&mut OsRng);
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(b"unique_nonce_");

    let plaintext = fs::read(input_path)?;
    let ciphertext = cipher.encrypt(nonce, plaintext.as_ref())
        .map_err(|e| format!("Encryption failed: {}", e))?;

    fs::write(output_path, ciphertext)?;
    println!("Encryption successful. Key: {}", hex::encode(key));
    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str, key_hex: &str) -> Result<(), Box<dyn std::error::Error>> {
    let key_bytes = hex::decode(key_hex)?;
    let key = key_bytes.as_slice().try_into()?;
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(b"unique_nonce_");

    let ciphertext = fs::read(input_path)?;
    let plaintext = cipher.decrypt(nonce, ciphertext.as_ref())
        .map_err(|e| format!("Decryption failed: {}", e))?;

    fs::write(output_path, plaintext)?;
    println!("Decryption successful.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_roundtrip() {
        let plaintext = b"Secret data for testing";
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        fs::write(input_file.path(), plaintext).unwrap();

        let key = hex::encode(Aes256Gcm::generate_key(&mut OsRng));
        encrypt_file(input_file.path().to_str().unwrap(), 
                    encrypted_file.path().to_str().unwrap()).unwrap();
        
        decrypt_file(encrypted_file.path().to_str().unwrap(),
                    decrypted_file.path().to_str().unwrap(),
                    &key).unwrap();

        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(plaintext.to_vec(), decrypted_data);
    }
}