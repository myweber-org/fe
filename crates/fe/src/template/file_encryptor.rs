
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use argon2::{
    password_hash::{rand_core::OsRng as ArgonRng, PasswordHasher, SaltString},
    Argon2, Params
};
use std::{
    fs,
    io::{self, Read, Write},
    path::Path
};

const SALT_LENGTH: usize = 16;
const NONCE_LENGTH: usize = 12;

pub struct FileEncryptor {
    algorithm: String,
    key_derivation: String,
}

impl FileEncryptor {
    pub fn new() -> Self {
        FileEncryptor {
            algorithm: "AES-256-GCM".to_string(),
            key_derivation: "Argon2id".to_string(),
        }
    }

    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path, password: &str) -> io::Result<()> {
        let mut input_file = fs::File::open(input_path)?;
        let mut plaintext = Vec::new();
        input_file.read_to_end(&mut plaintext)?;

        let salt = SaltString::generate(&mut ArgonRng);
        let argon2 = Argon2::new(
            argon2::Algorithm::Argon2id,
            argon2::Version::V0x13,
            Params::new(15000, 2, 1, Some(32)).unwrap(),
        );

        let password_hash = argon2.hash_password(password.as_bytes(), &salt).unwrap();
        let key_bytes = password_hash.hash.unwrap().as_bytes();
        let key = Key::<Aes256Gcm>::from_slice(key_bytes);

        let cipher = Aes256Gcm::new(key);
        let nonce = Nonce::from_slice(&[0u8; NONCE_LENGTH]);
        
        let ciphertext = cipher.encrypt(nonce, plaintext.as_ref())
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        let mut output_file = fs::File::create(output_path)?;
        output_file.write_all(salt.as_bytes())?;
        output_file.write_all(&ciphertext)?;

        Ok(())
    }

    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path, password: &str) -> io::Result<()> {
        let mut input_file = fs::File::open(input_path)?;
        let mut encrypted_data = Vec::new();
        input_file.read_to_end(&mut encrypted_data)?;

        if encrypted_data.len() < SALT_LENGTH {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "File too short"));
        }

        let salt_bytes = &encrypted_data[..SALT_LENGTH];
        let ciphertext = &encrypted_data[SALT_LENGTH..];
        let salt = SaltString::from_b64(std::str::from_utf8(salt_bytes).unwrap()).unwrap();

        let argon2 = Argon2::new(
            argon2::Algorithm::Argon2id,
            argon2::Version::V0x13,
            Params::new(15000, 2, 1, Some(32)).unwrap(),
        );

        let password_hash = argon2.hash_password(password.as_bytes(), &salt).unwrap();
        let key_bytes = password_hash.hash.unwrap().as_bytes();
        let key = Key::<Aes256Gcm>::from_slice(key_bytes);

        let cipher = Aes256Gcm::new(key);
        let nonce = Nonce::from_slice(&[0u8; NONCE_LENGTH]);
        
        let plaintext = cipher.decrypt(nonce, ciphertext)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        let mut output_file = fs::File::create(output_path)?;
        output_file.write_all(&plaintext)?;

        Ok(())
    }

    pub fn get_algorithm_info(&self) -> String {
        format!("{} with {} key derivation", self.algorithm, self.key_derivation)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let encryptor = FileEncryptor::new();
        let test_data = b"Secret data that needs protection";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        fs::write(input_file.path(), test_data).unwrap();

        encryptor.encrypt_file(input_file.path(), encrypted_file.path(), "strong_password")
            .expect("Encryption failed");

        encryptor.decrypt_file(encrypted_file.path(), decrypted_file.path(), "strong_password")
            .expect("Decryption failed");

        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(test_data.to_vec(), decrypted_data);
    }

    #[test]
    fn test_wrong_password_fails() {
        let encryptor = FileEncryptor::new();
        let test_data = b"Test content";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        fs::write(input_file.path(), test_data).unwrap();

        encryptor.encrypt_file(input_file.path(), encrypted_file.path(), "correct_password")
            .expect("Encryption failed");

        let result = encryptor.decrypt_file(encrypted_file.path(), decrypted_file.path(), "wrong_password");
        assert!(result.is_err());
    }
}