
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use argon2::{
    password_hash::{rand_core::OsRng as ArgonRng, PasswordHasher, SaltString},
    Argon2,
};
use std::{
    fs::{self, File},
    io::{Read, Write},
    path::Path,
};

const NONCE_SIZE: usize = 12;
const SALT_SIZE: usize = 16;

pub struct EncryptionResult {
    pub encrypted_data: Vec<u8>,
    pub nonce: [u8; NONCE_SIZE],
    pub salt: [u8; SALT_SIZE],
}

pub fn derive_key(password: &str, salt: &[u8]) -> Result<Key<Aes256Gcm>, String> {
    let salt_str = SaltString::encode_b64(salt).map_err(|e| e.to_string())?;
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt_str)
        .map_err(|e| e.to_string())?;

    let hash_bytes = password_hash.hash.ok_or("Hash generation failed")?;
    let key_bytes: [u8; 32] = hash_bytes.as_bytes()[..32]
        .try_into()
        .map_err(|_| "Hash too short")?;

    Ok(Key::<Aes256Gcm>::from_slice(&key_bytes).clone())
}

pub fn encrypt_file(
    input_path: &Path,
    output_path: &Path,
    password: &str,
) -> Result<EncryptionResult, String> {
    let mut file = File::open(input_path).map_err(|e| e.to_string())?;
    let mut plaintext = Vec::new();
    file.read_to_end(&mut plaintext).map_err(|e| e.to_string())?;

    let mut rng = OsRng;
    let mut nonce = [0u8; NONCE_SIZE];
    let mut salt = [0u8; SALT_SIZE];
    rng.fill_bytes(&mut nonce);
    rng.fill_bytes(&mut salt);

    let key = derive_key(password, &salt)?;
    let cipher = Aes256Gcm::new(&key);
    let nonce_obj = Nonce::from_slice(&nonce);

    let encrypted_data = cipher
        .encrypt(nonce_obj, plaintext.as_ref())
        .map_err(|e| e.to_string())?;

    let mut output_file = File::create(output_path).map_err(|e| e.to_string())?;
    output_file
        .write_all(&encrypted_data)
        .map_err(|e| e.to_string())?;

    Ok(EncryptionResult {
        encrypted_data,
        nonce,
        salt,
    })
}

pub fn decrypt_file(
    input_path: &Path,
    output_path: &Path,
    password: &str,
    nonce: &[u8; NONCE_SIZE],
    salt: &[u8; SALT_SIZE],
) -> Result<Vec<u8>, String> {
    let mut file = File::open(input_path).map_err(|e| e.to_string())?;
    let mut ciphertext = Vec::new();
    file.read_to_end(&mut ciphertext).map_err(|e| e.to_string())?;

    let key = derive_key(password, salt)?;
    let cipher = Aes256Gcm::new(&key);
    let nonce_obj = Nonce::from_slice(nonce);

    let decrypted_data = cipher
        .decrypt(nonce_obj, ciphertext.as_ref())
        .map_err(|e| e.to_string())?;

    let mut output_file = File::create(output_path).map_err(|e| e.to_string())?;
    output_file
        .write_all(&decrypted_data)
        .map_err(|e| e.to_string())?;

    Ok(decrypted_data)
}

pub fn encrypt_directory(
    dir_path: &Path,
    password: &str,
    output_dir: &Path,
) -> Result<Vec<EncryptionResult>, String> {
    if !dir_path.is_dir() {
        return Err("Provided path is not a directory".to_string());
    }

    fs::create_dir_all(output_dir).map_err(|e| e.to_string())?;
    let mut results = Vec::new();

    for entry in fs::read_dir(dir_path).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();

        if path.is_file() {
            let output_path = output_dir.join(path.file_name().ok_or("Invalid filename")?);
            let result = encrypt_file(&path, &output_path, password)?;
            results.push(result);
        }
    }

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let test_data = b"Test encryption data";
        let password = "secure_password_123";

        let mut input_file = NamedTempFile::new().unwrap();
        input_file.write_all(test_data).unwrap();

        let output_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        let encrypt_result = encrypt_file(
            input_file.path(),
            output_file.path(),
            password,
        )
        .unwrap();

        let decrypted_data = decrypt_file(
            output_file.path(),
            decrypted_file.path(),
            password,
            &encrypt_result.nonce,
            &encrypt_result.salt,
        )
        .unwrap();

        assert_eq!(decrypted_data, test_data);
    }

    #[test]
    fn test_wrong_password() {
        let test_data = b"Test data";
        let password = "correct_password";
        let wrong_password = "wrong_password";

        let mut input_file = NamedTempFile::new().unwrap();
        input_file.write_all(test_data).unwrap();

        let output_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        let encrypt_result = encrypt_file(
            input_file.path(),
            output_file.path(),
            password,
        )
        .unwrap();

        let result = decrypt_file(
            output_file.path(),
            decrypted_file.path(),
            wrong_password,
            &encrypt_result.nonce,
            &encrypt_result.salt,
        );

        assert!(result.is_err());
    }
}