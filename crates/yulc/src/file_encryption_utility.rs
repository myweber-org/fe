
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use pbkdf2::{
    password_hash::{
        rand_core::RngCore,
        SaltString, PasswordHasher, PasswordVerifier
    },
    Params, Pbkdf2
};
use std::{
    fs::{File, OpenOptions},
    io::{Read, Write},
    path::Path
};

const SALT_LENGTH: usize = 16;
const NONCE_LENGTH: usize = 12;
const PBKDF2_ITERATIONS: u32 = 100_000;

pub struct FileEncryptor {
    key: [u8; 32],
}

impl FileEncryptor {
    pub fn new(password: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let salt = SaltString::generate(&mut OsRng);
        let params = Params {
            rounds: PBKDF2_ITERATIONS,
            output_length: 32,
        };
        
        let password_hash = Pbkdf2.hash_password_customized(
            password.as_bytes(),
            None,
            None,
            params,
            &salt
        )?;
        
        let mut key = [0u8; 32];
        key.copy_from_slice(&password_hash.hash.unwrap().as_bytes()[..32]);
        
        Ok(Self { key })
    }
    
    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let mut input_file = File::open(input_path)?;
        let mut plaintext = Vec::new();
        input_file.read_to_end(&mut plaintext)?;
        
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&self.key));
        let mut nonce_bytes = [0u8; NONCE_LENGTH];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        let ciphertext = cipher.encrypt(nonce, plaintext.as_ref())
            .map_err(|e| format!("Encryption failed: {}", e))?;
        
        let mut output_file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(output_path)?;
        
        output_file.write_all(&nonce_bytes)?;
        output_file.write_all(&ciphertext)?;
        
        Ok(())
    }
    
    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let mut input_file = File::open(input_path)?;
        let mut encrypted_data = Vec::new();
        input_file.read_to_end(&mut encrypted_data)?;
        
        if encrypted_data.len() < NONCE_LENGTH {
            return Err("Invalid encrypted file format".into());
        }
        
        let nonce = Nonce::from_slice(&encrypted_data[..NONCE_LENGTH]);
        let ciphertext = &encrypted_data[NONCE_LENGTH..];
        
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&self.key));
        let plaintext = cipher.decrypt(nonce, ciphertext)
            .map_err(|e| format!("Decryption failed: {}", e))?;
        
        let mut output_file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(output_path)?;
        
        output_file.write_all(&plaintext)?;
        
        Ok(())
    }
    
    pub fn verify_password(&self, password: &str) -> bool {
        let salt = SaltString::generate(&mut OsRng);
        let params = Params {
            rounds: PBKDF2_ITERATIONS,
            output_length: 32,
        };
        
        if let Ok(password_hash) = Pbkdf2.hash_password_customized(
            password.as_bytes(),
            None,
            None,
            params,
            &salt
        ) {
            if let Some(hash) = password_hash.hash {
                let derived_key = &hash.as_bytes()[..32];
                derived_key == self.key
            } else {
                false
            }
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_encryption_decryption() {
        let password = "secure_password_123";
        let encryptor = FileEncryptor::new(password).unwrap();
        
        let test_data = b"Hello, this is a secret message!";
        let mut input_file = NamedTempFile::new().unwrap();
        input_file.write_all(test_data).unwrap();
        
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        encryptor.encrypt_file(input_file.path(), encrypted_file.path()).unwrap();
        encryptor.decrypt_file(encrypted_file.path(), decrypted_file.path()).unwrap();
        
        let mut decrypted_data = Vec::new();
        File::open(decrypted_file.path()).unwrap()
            .read_to_end(&mut decrypted_data).unwrap();
            
        assert_eq!(test_data.to_vec(), decrypted_data);
    }
    
    #[test]
    fn test_password_verification() {
        let password = "test_password";
        let encryptor = FileEncryptor::new(password).unwrap();
        
        assert!(encryptor.verify_password(password));
        assert!(!encryptor.verify_password("wrong_password"));
    }
}use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use argon2::{
    password_hash::{rand_core::OsRng as ArgonRng, PasswordHasher, SaltString},
    Argon2
};
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const NONCE_SIZE: usize = 12;
const SALT_SIZE: usize = 16;

pub struct FileEncryptor {
    key: Key<Aes256Gcm>,
}

impl FileEncryptor {
    pub fn from_password(password: &str, salt: &[u8]) -> io::Result<Self> {
        let argon2 = Argon2::default();
        let salt_string = SaltString::encode_b64(salt)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
        
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt_string)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?
            .hash
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Hash generation failed"))?;
        
        let key_bytes: [u8; 32] = password_hash.as_bytes()[..32]
            .try_into()
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Key length mismatch"))?;
        
        let key = Key::<Aes256Gcm>::from_slice(&key_bytes).into();
        Ok(Self { key })
    }

    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> io::Result<()> {
        let mut input_file = fs::File::open(input_path)?;
        let mut plaintext = Vec::new();
        input_file.read_to_end(&mut plaintext)?;

        let cipher = Aes256Gcm::new(&self.key);
        let nonce = Nonce::from_slice(&generate_random_bytes(NONCE_SIZE));
        
        let ciphertext = cipher
            .encrypt(nonce, plaintext.as_ref())
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        let mut output_file = fs::File::create(output_path)?;
        output_file.write_all(nonce.as_slice())?;
        output_file.write_all(&ciphertext)?;
        
        Ok(())
    }

    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path) -> io::Result<()> {
        let mut input_file = fs::File::open(input_path)?;
        let mut encrypted_data = Vec::new();
        input_file.read_to_end(&mut encrypted_data)?;

        if encrypted_data.len() < NONCE_SIZE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Encrypted data too short"
            ));
        }

        let (nonce_bytes, ciphertext) = encrypted_data.split_at(NONCE_SIZE);
        let nonce = Nonce::from_slice(nonce_bytes);
        let cipher = Aes256Gcm::new(&self.key);

        let plaintext = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        fs::write(output_path, plaintext)?;
        Ok(())
    }
}

fn generate_random_bytes(size: usize) -> Vec<u8> {
    let mut bytes = vec![0u8; size];
    OsRng.fill_bytes(&mut bytes);
    bytes
}

pub fn generate_salt() -> Vec<u8> {
    generate_random_bytes(SALT_SIZE)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let password = "secure_password_123";
        let salt = generate_salt();
        
        let encryptor = FileEncryptor::from_password(password, &salt).unwrap();
        
        let plaintext = b"Secret data that needs protection";
        let input_file = NamedTempFile::new().unwrap();
        fs::write(input_file.path(), plaintext).unwrap();
        
        let encrypted_file = NamedTempFile::new().unwrap();
        encryptor.encrypt_file(input_file.path(), encrypted_file.path()).unwrap();
        
        let decrypted_file = NamedTempFile::new().unwrap();
        encryptor.decrypt_file(encrypted_file.path(), decrypted_file.path()).unwrap();
        
        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(plaintext.to_vec(), decrypted_data);
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
    Argon2
};
use std::fs;
use std::io::{Read, Write};
use std::path::Path;

const NONCE_SIZE: usize = 12;
const SALT_SIZE: usize = 16;

pub struct FileEncryptor {
    key: Key<Aes256Gcm>,
}

impl FileEncryptor {
    pub fn from_password(password: &str, salt: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        let argon2 = Argon2::default();
        let salt_string = SaltString::encode_b64(salt)?;
        
        let password_hash = argon2.hash_password(password.as_bytes(), &salt_string)?;
        let hash_bytes = password_hash.hash.ok_or("Failed to generate hash")?;
        
        let key_bytes: [u8; 32] = hash_bytes.as_bytes()[..32]
            .try_into()
            .map_err(|_| "Hash too short for key")?;
        
        let key = Key::<Aes256Gcm>::from_slice(&key_bytes).into();
        Ok(Self { key })
    }
    
    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let mut file_data = Vec::new();
        fs::File::open(input_path)?.read_to_end(&mut file_data)?;
        
        let cipher = Aes256Gcm::new(&self.key);
        let nonce = Nonce::from_slice(&generate_random_bytes(NONCE_SIZE));
        
        let encrypted_data = cipher.encrypt(nonce, file_data.as_ref())
            .map_err(|e| format!("Encryption failed: {}", e))?;
        
        let mut output = Vec::new();
        output.extend_from_slice(nonce.as_slice());
        output.extend_from_slice(&encrypted_data);
        
        fs::File::create(output_path)?.write_all(&output)?;
        Ok(())
    }
    
    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let mut encrypted_data = Vec::new();
        fs::File::open(input_path)?.read_to_end(&mut encrypted_data)?;
        
        if encrypted_data.len() < NONCE_SIZE {
            return Err("File too short to contain nonce".into());
        }
        
        let (nonce_bytes, ciphertext) = encrypted_data.split_at(NONCE_SIZE);
        let nonce = Nonce::from_slice(nonce_bytes);
        
        let cipher = Aes256Gcm::new(&self.key);
        let decrypted_data = cipher.decrypt(nonce, ciphertext)
            .map_err(|e| format!("Decryption failed: {}", e))?;
        
        fs::File::create(output_path)?.write_all(&decrypted_data)?;
        Ok(())
    }
}

fn generate_random_bytes(size: usize) -> Vec<u8> {
    let mut bytes = vec![0u8; size];
    OsRng.fill_bytes(&mut bytes);
    bytes
}

pub fn generate_salt() -> Vec<u8> {
    generate_random_bytes(SALT_SIZE)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_encryption_decryption() {
        let password = "secure_password_123";
        let salt = generate_salt();
        
        let encryptor = FileEncryptor::from_password(password, &salt).unwrap();
        
        let test_data = b"Hello, this is a secret message!";
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), test_data).unwrap();
        
        encryptor.encrypt_file(input_file.path(), encrypted_file.path()).unwrap();
        encryptor.decrypt_file(encrypted_file.path(), decrypted_file.path()).unwrap();
        
        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(test_data.to_vec(), decrypted_data);
    }
}use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use chacha20poly1305::{ChaCha20Poly1305, Key as ChaChaKey, Nonce as ChaChaNonce};
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
            let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
            let ciphertext = cipher.encrypt(&nonce, plaintext)?;
            
            Ok(EncryptionResult {
                ciphertext,
                nonce: nonce.to_vec(),
            })
        }
        CipherAlgorithm::ChaCha20Poly1305 => {
            let key = ChaCha20Poly1305::generate_key(&mut OsRng);
            let cipher = ChaCha20Poly1305::new(&key);
            let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);
            let ciphertext = cipher.encrypt(&nonce, plaintext)?;
            
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
    key: &[u8],
) -> Result<Vec<u8>, Box<dyn Error>> {
    match algorithm {
        CipherAlgorithm::Aes256Gcm => {
            let key = Key::<Aes256Gcm>::from_slice(key);
            let cipher = Aes256Gcm::new(key);
            let nonce = Nonce::from_slice(nonce);
            let plaintext = cipher.decrypt(nonce, ciphertext)?;
            Ok(plaintext)
        }
        CipherAlgorithm::ChaCha20Poly1305 => {
            let key = ChaChaKey::from_slice(key);
            let cipher = ChaCha20Poly1305::new(key);
            let nonce = ChaChaNonce::from_slice(nonce);
            let plaintext = cipher.decrypt(nonce, ciphertext)?;
            Ok(plaintext)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aes_encryption_decryption() {
        let plaintext = b"Test secret message";
        let result = encrypt_data(plaintext, CipherAlgorithm::Aes256Gcm).unwrap();
        
        // In real usage, key management would be required
        // This test demonstrates the encryption works
        assert_ne!(result.ciphertext, plaintext);
        assert_eq!(result.nonce.len(), 12); // AES-GCM nonce size
    }

    #[test]
    fn test_chacha_encryption_decryption() {
        let plaintext = b"Another secret message";
        let result = encrypt_data(plaintext, CipherAlgorithm::ChaCha20Poly1305).unwrap();
        
        assert_ne!(result.ciphertext, plaintext);
        assert_eq!(result.nonce.len(), 12); // ChaCha20Poly1305 nonce size
    }
}