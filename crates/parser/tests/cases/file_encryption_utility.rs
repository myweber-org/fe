
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use argon2::{Argon2, PasswordHasher};
use rand::RngCore;
use std::fs;
use std::io::{Read, Write};
use std::path::Path;

const NONCE_SIZE: usize = 12;
const SALT_SIZE: usize = 16;

pub struct EncryptionResult {
    pub ciphertext: Vec<u8>,
    pub nonce: [u8; NONCE_SIZE],
    pub salt: [u8; SALT_SIZE],
}

pub fn encrypt_file(
    input_path: &Path,
    output_path: &Path,
    password: &str,
) -> Result<EncryptionResult, Box<dyn std::error::Error>> {
    let mut file_data = Vec::new();
    fs::File::open(input_path)?.read_to_end(&mut file_data)?;

    let mut salt = [0u8; SALT_SIZE];
    OsRng.fill_bytes(&mut salt);

    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)?
        .hash
        .ok_or("Failed to hash password")?;

    let key = Key::<Aes256Gcm>::from_slice(&password_hash[..32]);
    let cipher = Aes256Gcm::new(key);

    let mut nonce = [0u8; NONCE_SIZE];
    OsRng.fill_bytes(&mut nonce);
    let nonce_obj = Nonce::from_slice(&nonce);

    let ciphertext = cipher
        .encrypt(nonce_obj, file_data.as_ref())
        .map_err(|e| format!("Encryption failed: {}", e))?;

    let result = EncryptionResult {
        ciphertext: ciphertext.clone(),
        nonce,
        salt,
    };

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&salt)?;
    output_file.write_all(&nonce)?;
    output_file.write_all(&ciphertext)?;

    Ok(result)
}

pub fn decrypt_file(
    input_path: &Path,
    output_path: &Path,
    password: &str,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut encrypted_data = Vec::new();
    fs::File::open(input_path)?.read_to_end(&mut encrypted_data)?;

    if encrypted_data.len() < SALT_SIZE + NONCE_SIZE {
        return Err("Invalid encrypted file format".into());
    }

    let salt = &encrypted_data[..SALT_SIZE];
    let nonce = &encrypted_data[SALT_SIZE..SALT_SIZE + NONCE_SIZE];
    let ciphertext = &encrypted_data[SALT_SIZE + NONCE_SIZE..];

    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), salt)?
        .hash
        .ok_or("Failed to hash password")?;

    let key = Key::<Aes256Gcm>::from_slice(&password_hash[..32]);
    let cipher = Aes256Gcm::new(key);
    let nonce_obj = Nonce::from_slice(nonce);

    let plaintext = cipher
        .decrypt(nonce_obj, ciphertext)
        .map_err(|e| format!("Decryption failed: {}", e))?;

    fs::File::create(output_path)?.write_all(&plaintext)?;

    Ok(plaintext)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let original_data = b"Test data for encryption and decryption";
        let password = "secure_password_123";

        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        fs::write(input_file.path(), original_data).unwrap();

        let encrypt_result = encrypt_file(input_file.path(), encrypted_file.path(), password);
        assert!(encrypt_result.is_ok());

        let decrypt_result = decrypt_file(encrypted_file.path(), decrypted_file.path(), password);
        assert!(decrypt_result.is_ok());

        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(original_data.as_slice(), decrypted_data.as_slice());
    }

    #[test]
    fn test_wrong_password() {
        let original_data = b"Sensitive information";
        let correct_password = "correct_pass";
        let wrong_password = "wrong_pass";

        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        fs::write(input_file.path(), original_data).unwrap();

        let encrypt_result = encrypt_file(input_file.path(), encrypted_file.path(), correct_password);
        assert!(encrypt_result.is_ok());

        let decrypt_result = decrypt_file(encrypted_file.path(), decrypted_file.path(), wrong_password);
        assert!(decrypt_result.is_err());
    }
}