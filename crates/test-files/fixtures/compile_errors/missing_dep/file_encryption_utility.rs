use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use argon2::{
    password_hash::{rand_core::OsRng as ArgonRng, PasswordHasher, SaltString},
    Argon2, Params,
};
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const SALT_LENGTH: usize = 16;
const NONCE_LENGTH: usize = 12;

pub struct EncryptionResult {
    pub ciphertext: Vec<u8>,
    pub nonce: [u8; NONCE_LENGTH],
    pub salt: [u8; SALT_LENGTH],
}

pub fn derive_key(password: &str, salt: &[u8]) -> Result<Key<Aes256Gcm>, String> {
    let salt_str = SaltString::encode_b64(salt).map_err(|e| e.to_string())?;
    let argon2 = Argon2::new(
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        Params::new(15000, 2, 1, Some(32)).map_err(|e| e.to_string())?,
    );

    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt_str)
        .map_err(|e| e.to_string())?;

    let key_bytes = password_hash.hash.ok_or("Hash generation failed")?;
    let key_slice: &[u8; 32] = key_bytes
        .as_bytes()
        .try_into()
        .map_err(|_| "Invalid key length")?;

    Ok(*Key::<Aes256Gcm>::from_slice(key_slice))
}

pub fn encrypt_file(
    input_path: &Path,
    output_path: &Path,
    password: &str,
) -> Result<EncryptionResult, String> {
    let mut file = fs::File::open(input_path).map_err(|e| e.to_string())?;
    let mut plaintext = Vec::new();
    file.read_to_end(&mut plaintext).map_err(|e| e.to_string())?;

    let mut salt = [0u8; SALT_LENGTH];
    ArgonRng.fill_bytes(&mut salt);

    let key = derive_key(password, &salt)?;

    let cipher = Aes256Gcm::new(&key);
    let mut nonce_bytes = [0u8; NONCE_LENGTH];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_ref())
        .map_err(|e| e.to_string())?;

    let mut output_file = fs::File::create(output_path).map_err(|e| e.to_string())?;
    output_file
        .write_all(&ciphertext)
        .map_err(|e| e.to_string())?;

    Ok(EncryptionResult {
        ciphertext,
        nonce: nonce_bytes,
        salt,
    })
}

pub fn decrypt_file(
    input_path: &Path,
    output_path: &Path,
    password: &str,
    nonce: &[u8; NONCE_LENGTH],
    salt: &[u8; SALT_LENGTH],
) -> Result<Vec<u8>, String> {
    let mut file = fs::File::open(input_path).map_err(|e| e.to_string())?;
    let mut ciphertext = Vec::new();
    file.read_to_end(&mut ciphertext).map_err(|e| e.to_string())?;

    let key = derive_key(password, salt)?;
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(nonce);

    let plaintext = cipher
        .decrypt(nonce, ciphertext.as_ref())
        .map_err(|e| e.to_string())?;

    fs::write(output_path, &plaintext).map_err(|e| e.to_string())?;
    Ok(plaintext)
}

pub fn generate_key_file(output_path: &Path) -> Result<(), String> {
    let mut key_bytes = [0u8; 32];
    OsRng.fill_bytes(&mut key_bytes);

    let mut file = fs::File::create(output_path).map_err(|e| e.to_string())?;
    file.write_all(&key_bytes).map_err(|e| e.to_string())?;

    println!("Key saved to: {}", output_path.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let plaintext = b"Test data for encryption";
        let password = "secure_password_123";

        let mut input_file = NamedTempFile::new().unwrap();
        input_file.write_all(plaintext).unwrap();

        let output_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        let result = encrypt_file(input_file.path(), output_file.path(), password).unwrap();

        let decrypted = decrypt_file(
            output_file.path(),
            decrypted_file.path(),
            password,
            &result.nonce,
            &result.salt,
        )
        .unwrap();

        assert_eq!(decrypted, plaintext);
    }
}use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use std::error::Error;

pub struct EncryptionResult {
    pub ciphertext: Vec<u8>,
    pub nonce: Vec<u8>,
}

pub fn encrypt_data(plaintext: &[u8], key: &[u8]) -> Result<EncryptionResult, Box<dyn Error>> {
    if key.len() != 32 {
        return Err("Key must be 32 bytes for AES-256".into());
    }

    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let nonce_bytes: [u8; 12] = OsRng.fill(&mut [0u8; 12])?;
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| format!("Encryption failed: {}", e))?;

    Ok(EncryptionResult {
        ciphertext,
        nonce: nonce_bytes.to_vec(),
    })
}

pub fn decrypt_data(
    ciphertext: &[u8],
    key: &[u8],
    nonce: &[u8],
) -> Result<Vec<u8>, Box<dyn Error>> {
    if key.len() != 32 {
        return Err("Key must be 32 bytes for AES-256".into());
    }
    if nonce.len() != 12 {
        return Err("Nonce must be 12 bytes".into());
    }

    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let nonce = Nonce::from_slice(nonce);

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| format!("Decryption failed: {}", e))?;

    Ok(plaintext)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_roundtrip() {
        let key = [0x42u8; 32];
        let plaintext = b"Secret message for encryption test";
        
        let result = encrypt_data(plaintext, &key).unwrap();
        let decrypted = decrypt_data(&result.ciphertext, &key, &result.nonce).unwrap();
        
        assert_eq!(plaintext.to_vec(), decrypted);
    }

    #[test]
    fn test_invalid_key_length() {
        let short_key = [0x42u8; 16];
        let plaintext = b"Test";
        
        assert!(encrypt_data(plaintext, &short_key).is_err());
    }
}