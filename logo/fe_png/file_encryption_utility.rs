use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use std::{
    fs::{self, File},
    io::{Read, Write},
    path::Path,
};

const NONCE_SIZE: usize = 12;

pub fn encrypt_file(input_path: &Path, output_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let key = Aes256Gcm::generate_key(&mut OsRng);
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(&[0u8; NONCE_SIZE]);

    let mut input_file = File::open(input_path)?;
    let mut plaintext = Vec::new();
    input_file.read_to_end(&mut plaintext)?;

    let ciphertext = cipher.encrypt(nonce, plaintext.as_ref())
        .map_err(|e| format!("Encryption failed: {}", e))?;

    let mut output_file = File::create(output_path)?;
    output_file.write_all(&key)?;
    output_file.write_all(nonce)?;
    output_file.write_all(&ciphertext)?;

    Ok(())
}

pub fn decrypt_file(input_path: &Path, output_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let mut input_file = File::open(input_path)?;
    let mut encrypted_data = Vec::new();
    input_file.read_to_end(&mut encrypted_data)?;

    if encrypted_data.len() < 32 + NONCE_SIZE {
        return Err("Invalid encrypted file format".into());
    }

    let key = Key::<Aes256Gcm>::from_slice(&encrypted_data[..32]);
    let nonce = Nonce::from_slice(&encrypted_data[32..32 + NONCE_SIZE]);
    let ciphertext = &encrypted_data[32 + NONCE_SIZE..];

    let cipher = Aes256Gcm::new(key);
    let plaintext = cipher.decrypt(nonce, ciphertext)
        .map_err(|e| format!("Decryption failed: {}", e))?;

    let mut output_file = File::create(output_path)?;
    output_file.write_all(&plaintext)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let original_content = b"Secret data that needs protection";
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        fs::write(input_file.path(), original_content).unwrap();

        encrypt_file(input_file.path(), encrypted_file.path()).unwrap();
        decrypt_file(encrypted_file.path(), decrypted_file.path()).unwrap();

        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(original_content.to_vec(), decrypted_content);
    }
}use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use argon2::{
    password_hash::{rand_core::OsRng as ArgonRng, PasswordHasher, SaltString},
    Argon2,
};
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const NONCE_SIZE: usize = 12;

pub struct EncryptionResult {
    pub ciphertext: Vec<u8>,
    pub nonce: [u8; NONCE_SIZE],
}

pub fn derive_key(password: &str, salt: &SaltString) -> io::Result<Key<Aes256Gcm>> {
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), salt)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

    let hash_bytes = password_hash
        .hash
        .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Hash generation failed"))?
        .as_bytes();

    if hash_bytes.len() < 32 {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "Insufficient hash length",
        ));
    }

    let key_slice = &hash_bytes[..32];
    Ok(*Key::<Aes256Gcm>::from_slice(key_slice))
}

pub fn encrypt_data(data: &[u8], password: &str) -> io::Result<EncryptionResult> {
    let salt = SaltString::generate(&mut ArgonRng);
    let key = derive_key(password, &salt)?;
    let cipher = Aes256Gcm::new(&key);

    let mut nonce_bytes = [0u8; NONCE_SIZE];
    OsRng.fill(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let mut combined_data = salt.as_bytes().to_vec();
    combined_data.extend_from_slice(data);

    let ciphertext = cipher
        .encrypt(nonce, combined_data.as_ref())
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

    Ok(EncryptionResult {
        ciphertext,
        nonce: nonce_bytes,
    })
}

pub fn decrypt_data(
    ciphertext: &[u8],
    nonce: &[u8; NONCE_SIZE],
    password: &str,
) -> io::Result<Vec<u8>> {
    if ciphertext.len() < 22 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Ciphertext too short",
        ));
    }

    let salt = SaltString::from_b64(&base64::encode(&ciphertext[..22]))
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

    let key = derive_key(password, &salt)?;
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(nonce);

    let plaintext = cipher
        .decrypt(nonce, &ciphertext[22..])
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

    Ok(plaintext[22..].to_vec())
}

pub fn encrypt_file(input_path: &Path, output_path: &Path, password: &str) -> io::Result<()> {
    let mut file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    let result = encrypt_data(&buffer, password)?;

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&result.nonce)?;
    output_file.write_all(&result.ciphertext)?;

    Ok(())
}

pub fn decrypt_file(input_path: &Path, output_path: &Path, password: &str) -> io::Result<()> {
    let mut file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    if buffer.len() < NONCE_SIZE {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "File too small to contain nonce",
        ));
    }

    let nonce: [u8; NONCE_SIZE] = buffer[..NONCE_SIZE].try_into().unwrap();
    let ciphertext = &buffer[NONCE_SIZE..];

    let decrypted_data = decrypt_data(ciphertext, &nonce, password)?;

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&decrypted_data)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_roundtrip() {
        let test_data = b"Secret data that needs protection";
        let password = "strong_password_123";

        let encrypted = encrypt_data(test_data, password).unwrap();
        let decrypted = decrypt_data(&encrypted.ciphertext, &encrypted.nonce, password).unwrap();

        assert_eq!(test_data.to_vec(), decrypted);
    }

    #[test]
    fn test_file_encryption() {
        let test_content = b"File content for encryption test";
        let password = "test_password";

        let input_file = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        fs::write(input_file.path(), test_content).unwrap();

        encrypt_file(input_file.path(), output_file.path(), password).unwrap();
        decrypt_file(output_file.path(), decrypted_file.path(), password).unwrap();

        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(test_content.to_vec(), decrypted_content);
    }

    #[test]
    fn test_wrong_password_fails() {
        let test_data = b"Sensitive information";
        let password = "correct_password";

        let encrypted = encrypt_data(test_data, password).unwrap();
        let result = decrypt_data(&encrypted.ciphertext, &encrypted.nonce, "wrong_password");

        assert!(result.is_err());
    }
}