use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use argon2::{
    password_hash::{rand_core::OsRng as ArgonRng, PasswordHasher, SaltString},
    Argon2,
};
use std::fs;
use std::io::{self, Read, Write};

const NONCE_SIZE: usize = 12;
const SALT_SIZE: usize = 16;

pub fn encrypt_file(input_path: &str, output_path: &str, password: &str) -> io::Result<()> {
    let mut file_data = Vec::new();
    fs::File::open(input_path)?.read_to_end(&mut file_data)?;

    let salt = SaltString::generate(&mut ArgonRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let key = Key::<Aes256Gcm>::from_slice(&password_hash.hash.unwrap().as_bytes()[..32]);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(&OsRng.gen::<[u8; NONCE_SIZE]>());

    let encrypted_data = cipher
        .encrypt(nonce, file_data.as_ref())
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let mut output = Vec::new();
    output.extend_from_slice(salt.as_bytes());
    output.extend_from_slice(nonce.as_slice());
    output.extend_from_slice(&encrypted_data);

    fs::File::create(output_path)?.write_all(&output)?;
    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str, password: &str) -> io::Result<()> {
    let mut encrypted_data = Vec::new();
    fs::File::open(input_path)?.read_to_end(&mut encrypted_data)?;

    if encrypted_data.len() < SALT_SIZE + NONCE_SIZE {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "File too short to contain salt and nonce",
        ));
    }

    let salt_bytes = &encrypted_data[..SALT_SIZE];
    let nonce_bytes = &encrypted_data[SALT_SIZE..SALT_SIZE + NONCE_SIZE];
    let ciphertext = &encrypted_data[SALT_SIZE + NONCE_SIZE..];

    let salt = SaltString::from_b64(std::str::from_utf8(salt_bytes).map_err(|_| {
        io::Error::new(io::ErrorKind::InvalidData, "Invalid salt encoding")
    })?)
    .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let key = Key::<Aes256Gcm>::from_slice(&password_hash.hash.unwrap().as_bytes()[..32]);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(nonce_bytes);

    let decrypted_data = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    fs::File::create(output_path)?.write_all(&decrypted_data)?;
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

        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(decrypted_content, plaintext);
    }
}