use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use pbkdf2::{
    password_hash::{PasswordHasher, SaltString},
    Pbkdf2,
};
use std::fs;
use std::io::{self, Read, Write};

const SALT_LENGTH: usize = 16;
const NONCE_LENGTH: usize = 12;

pub fn encrypt_file(input_path: &str, output_path: &str, password: &str) -> io::Result<()> {
    let mut input_file = fs::File::open(input_path)?;
    let mut plaintext = Vec::new();
    input_file.read_to_end(&mut plaintext)?;

    let salt = SaltString::generate(&mut OsRng);
    let key_material = Pbkdf2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?
        .hash
        .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Hash generation failed"))?;

    let key = Key::<Aes256Gcm>::from_slice(&key_material.as_bytes()[..32]);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(&OsRng.gen::<[u8; NONCE_LENGTH]>());

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_ref())
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(salt.as_bytes())?;
    output_file.write_all(nonce.as_slice())?;
    output_file.write_all(&ciphertext)?;

    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str, password: &str) -> io::Result<()> {
    let mut input_file = fs::File::open(input_path)?;
    let mut encrypted_data = Vec::new();
    input_file.read_to_end(&mut encrypted_data)?;

    if encrypted_data.len() < SALT_LENGTH + NONCE_LENGTH {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "File too short to contain salt and nonce",
        ));
    }

    let salt = SaltString::from_b64(
        std::str::from_utf8(&encrypted_data[..SALT_LENGTH])
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?,
    )
    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    let key_material = Pbkdf2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?
        .hash
        .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Hash generation failed"))?;

    let key = Key::<Aes256Gcm>::from_slice(&key_material.as_bytes()[..32]);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(&encrypted_data[SALT_LENGTH..SALT_LENGTH + NONCE_LENGTH]);

    let plaintext = cipher
        .decrypt(
            nonce,
            encrypted_data[SALT_LENGTH + NONCE_LENGTH..].as_ref(),
        )
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&plaintext)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let plaintext = b"Secret data that needs protection";
        let password = "strong_password_123";

        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        fs::write(input_file.path(), plaintext).unwrap();

        encrypt_file(
            input_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap(),
            password,
        )
        .unwrap();

        decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap(),
            password,
        )
        .unwrap();

        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(plaintext.to_vec(), decrypted_data);
    }

    #[test]
    fn test_wrong_password_fails() {
        let plaintext = b"Test data";
        let password = "correct_password";
        let wrong_password = "wrong_password";

        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        fs::write(input_file.path(), plaintext).unwrap();

        encrypt_file(
            input_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap(),
            password,
        )
        .unwrap();

        let result = decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap(),
            wrong_password,
        );

        assert!(result.is_err());
    }
}