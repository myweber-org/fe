use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use std::fs::{self, File};
use std::io::{Read, Write};

const NONCE_SIZE: usize = 12;

pub fn encrypt_file(input_path: &str, output_path: &str, password: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::open(input_path)?;
    let mut plaintext = Vec::new();
    file.read_to_end(&mut plaintext)?;

    let key = derive_key(password);
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(&generate_nonce());

    let ciphertext = cipher.encrypt(nonce, plaintext.as_ref())
        .map_err(|e| format!("Encryption failed: {}", e))?;

    let mut output_file = File::create(output_path)?;
    output_file.write_all(nonce.as_slice())?;
    output_file.write_all(&ciphertext)?;

    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str, password: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::open(input_path)?;
    let mut data = Vec::new();
    file.read_to_end(&mut data)?;

    if data.len() < NONCE_SIZE {
        return Err("File too short to contain nonce".into());
    }

    let (nonce_slice, ciphertext) = data.split_at(NONCE_SIZE);
    let nonce = Nonce::from_slice(nonce_slice);
    let key = derive_key(password);
    let cipher = Aes256Gcm::new(&key);

    let plaintext = cipher.decrypt(nonce, ciphertext)
        .map_err(|e| format!("Decryption failed: {}", e))?;

    let mut output_file = File::create(output_path)?;
    output_file.write_all(&plaintext)?;

    Ok(())
}

fn derive_key(password: &str) -> Key<Aes256Gcm> {
    let mut key = [0u8; 32];
    let password_bytes = password.as_bytes();
    for (i, byte) in password_bytes.iter().cycle().take(32).enumerate() {
        key[i] = *byte;
    }
    *Key::<Aes256Gcm>::from_slice(&key)
}

fn generate_nonce() -> [u8; NONCE_SIZE] {
    let mut nonce = [0u8; NONCE_SIZE];
    OsRng.fill_bytes(&mut nonce);
    nonce
}use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce as ChaChaNonce};
use std::error::Error;

pub enum CipherAlgorithm {
    Aes256Gcm,
    ChaCha20Poly1305,
}

pub struct EncryptionResult {
    pub ciphertext: Vec<u8>,
    pub nonce: Vec<u8>,
}

pub fn encrypt_data(
    plaintext: &[u8],
    algorithm: CipherAlgorithm,
) -> Result<EncryptionResult, Box<dyn Error>> {
    match algorithm {
        CipherAlgorithm::Aes256Gcm => {
            let key = Aes256Gcm::generate_key(&mut OsRng);
            let cipher = Aes256Gcm::new(&key);
            let nonce = Nonce::from_slice(&[0u8; 12]);
            let ciphertext = cipher.encrypt(nonce, plaintext)?;
            Ok(EncryptionResult {
                ciphertext,
                nonce: nonce.to_vec(),
            })
        }
        CipherAlgorithm::ChaCha20Poly1305 => {
            let key = ChaCha20Poly1305::generate_key(&mut OsRng);
            let cipher = ChaCha20Poly1305::new(&key);
            let nonce = ChaChaNonce::from_slice(&[0u8; 12]);
            let ciphertext = cipher.encrypt(nonce, plaintext)?;
            Ok(EncryptionResult {
                ciphertext,
                nonce: nonce.to_vec(),
            })
        }
    }
}

pub fn decrypt_data(
    ciphertext: &[u8],
    nonce: &[u8],
    algorithm: CipherAlgorithm,
) -> Result<Vec<u8>, Box<dyn Error>> {
    match algorithm {
        CipherAlgorithm::Aes256Gcm => {
            let key = Aes256Gcm::generate_key(&mut OsRng);
            let cipher = Aes256Gcm::new(&key);
            let nonce = Nonce::from_slice(nonce);
            cipher.decrypt(nonce, ciphertext).map_err(|e| e.into())
        }
        CipherAlgorithm::ChaCha20Poly1305 => {
            let key = ChaCha20Poly1305::generate_key(&mut OsRng);
            let cipher = ChaCha20Poly1305::new(&key);
            let nonce = ChaChaNonce::from_slice(nonce);
            cipher.decrypt(nonce, ciphertext).map_err(|e| e.into())
        }
    }
}use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHasher, SaltString
    },
    Argon2, PasswordHash, PasswordVerifier
};
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const NONCE_SIZE: usize = 12;
const SALT_SIZE: usize = 16;

pub struct FileEncryptor {
    cipher: Aes256Gcm,
}

impl FileEncryptor {
    pub fn new(password: &str, salt: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        let argon2 = Argon2::default();
        let mut key = [0u8; 32];
        
        argon2.hash_password_into(
            password.as_bytes(),
            salt,
            &mut key
        )?;
        
        let cipher_key = Key::<Aes256Gcm>::from_slice(&key);
        let cipher = Aes256Gcm::new(cipher_key);
        
        Ok(FileEncryptor { cipher })
    }
    
    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let mut input_file = fs::File::open(input_path)?;
        let mut plaintext = Vec::new();
        input_file.read_to_end(&mut plaintext)?;
        
        let mut nonce = [0u8; NONCE_SIZE];
        OsRng.fill_bytes(&mut nonce);
        
        let ciphertext = self.cipher.encrypt(Nonce::from_slice(&nonce), plaintext.as_ref())
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        
        let mut output_file = fs::File::create(output_path)?;
        output_file.write_all(&nonce)?;
        output_file.write_all(&ciphertext)?;
        
        Ok(())
    }
    
    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let mut input_file = fs::File::open(input_path)?;
        let mut encrypted_data = Vec::new();
        input_file.read_to_end(&mut encrypted_data)?;
        
        if encrypted_data.len() < NONCE_SIZE {
            return Err("Invalid encrypted file format".into());
        }
        
        let (nonce_bytes, ciphertext) = encrypted_data.split_at(NONCE_SIZE);
        let nonce = Nonce::from_slice(nonce_bytes);
        
        let plaintext = self.cipher.decrypt(nonce, ciphertext)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        
        let mut output_file = fs::File::create(output_path)?;
        output_file.write_all(&plaintext)?;
        
        Ok(())
    }
}

pub fn generate_salt() -> Vec<u8> {
    let mut salt = vec![0u8; SALT_SIZE];
    OsRng.fill_bytes(&mut salt);
    salt
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_encryption_decryption() {
        let password = "secure_password_123";
        let salt = generate_salt();
        
        let encryptor = FileEncryptor::new(password, &salt).unwrap();
        
        let original_content = b"Hello, this is a secret message!";
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), original_content).unwrap();
        
        encryptor.encrypt_file(input_file.path(), encrypted_file.path()).unwrap();
        encryptor.decrypt_file(encrypted_file.path(), decrypted_file.path()).unwrap();
        
        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(original_content.to_vec(), decrypted_content);
    }
}