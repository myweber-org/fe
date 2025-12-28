use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use argon2::{Argon2, PasswordHasher};
use rand::RngCore;
use std::fs;
use std::io::{Read, Write};

const NONCE_SIZE: usize = 12;
const SALT_SIZE: usize = 16;

pub struct FileEncryptor {
    key: [u8; 32],
}

impl FileEncryptor {
    pub fn new(password: &str, salt: &[u8; SALT_SIZE]) -> Self {
        let argon2 = Argon2::default();
        let mut key = [0u8; 32];
        argon2
            .hash_password_into(password.as_bytes(), salt, &mut key)
            .expect("Key derivation failed");
        Self { key }
    }

    pub fn encrypt_file(&self, input_path: &str, output_path: &str) -> Result<(), String> {
        let mut file_content = Vec::new();
        fs::File::open(input_path)
            .map_err(|e| format!("Failed to open input file: {}", e))?
            .read_to_end(&mut file_content)
            .map_err(|e| format!("Failed to read file: {}", e))?;

        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&self.key));
        let mut nonce = [0u8; NONCE_SIZE];
        OsRng.fill_bytes(&mut nonce);

        let encrypted_data = cipher
            .encrypt(Nonce::from_slice(&nonce), file_content.as_ref())
            .map_err(|e| format!("Encryption failed: {}", e))?;

        let mut output = fs::File::create(output_path)
            .map_err(|e| format!("Failed to create output file: {}", e))?;

        output
            .write_all(&nonce)
            .map_err(|e| format!("Failed to write nonce: {}", e))?;
        output
            .write_all(&encrypted_data)
            .map_err(|e| format!("Failed to write encrypted data: {}", e))?;

        Ok(())
    }

    pub fn decrypt_file(&self, input_path: &str, output_path: &str) -> Result<(), String> {
        let mut encrypted_content = Vec::new();
        fs::File::open(input_path)
            .map_err(|e| format!("Failed to open encrypted file: {}", e))?
            .read_to_end(&mut encrypted_content)
            .map_err(|e| format!("Failed to read encrypted file: {}", e))?;

        if encrypted_content.len() < NONCE_SIZE {
            return Err("File too short to contain nonce".to_string());
        }

        let (nonce_bytes, ciphertext) = encrypted_content.split_at(NONCE_SIZE);
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&self.key));

        let decrypted_data = cipher
            .decrypt(Nonce::from_slice(nonce_bytes), ciphertext)
            .map_err(|e| format!("Decryption failed: {}", e))?;

        fs::write(output_path, decrypted_data)
            .map_err(|e| format!("Failed to write decrypted file: {}", e))?;

        Ok(())
    }
}

pub fn generate_salt() -> [u8; SALT_SIZE] {
    let mut salt = [0u8; SALT_SIZE];
    OsRng.fill_bytes(&mut salt);
    salt
}