
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use pbkdf2::{
    password_hash::{
        rand_core::RngCore,
        PasswordHasher, SaltString
    },
    Params, Pbkdf2
};
use std::fs;
use std::io::{Read, Write};
use std::path::Path;

const SALT_LENGTH: usize = 16;
const NONCE_LENGTH: usize = 12;
const PBKDF2_ITERATIONS: u32 = 100_000;

pub struct FileEncryptor {
    cipher: Aes256Gcm,
}

impl FileEncryptor {
    pub fn from_password(password: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let salt = SaltString::generate(&mut OsRng);
        let params = Params {
            rounds: PBKDF2_ITERATIONS,
            output_length: 32,
        };
        
        let password_hash = Pbkdf2
            .hash_password_customized(
                password.as_bytes(),
                None,
                None,
                params,
                salt.as_salt()
            )?
            .hash
            .ok_or("Failed to generate password hash")?;
        
        let key = Key::<Aes256Gcm>::from_slice(password_hash.as_bytes());
        let cipher = Aes256Gcm::new(key);
        
        Ok(Self { cipher })
    }
    
    pub fn encrypt_file(
        &self,
        input_path: &Path,
        output_path: &Path
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut file_data = Vec::new();
        fs::File::open(input_path)?.read_to_end(&mut file_data)?;
        
        let mut nonce_bytes = [0u8; NONCE_LENGTH];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        let ciphertext = self.cipher
            .encrypt(nonce, file_data.as_ref())
            .map_err(|e| format!("Encryption failed: {}", e))?;
        
        let mut output_data = Vec::with_capacity(NONCE_LENGTH + ciphertext.len());
        output_data.extend_from_slice(&nonce_bytes);
        output_data.extend_from_slice(&ciphertext);
        
        fs::File::create(output_path)?.write_all(&output_data)?;
        
        Ok(())
    }
    
    pub fn decrypt_file(
        &self,
        input_path: &Path,
        output_path: &Path
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut encrypted_data = Vec::new();
        fs::File::open(input_path)?.read_to_end(&mut encrypted_data)?;
        
        if encrypted_data.len() < NONCE_LENGTH {
            return Err("Invalid encrypted file format".into());
        }
        
        let nonce = Nonce::from_slice(&encrypted_data[..NONCE_LENGTH]);
        let ciphertext = &encrypted_data[NONCE_LENGTH..];
        
        let plaintext = self.cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| format!("Decryption failed: {}", e))?;
        
        fs::File::create(output_path)?.write_all(&plaintext)?;
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_encryption_decryption() {
        let test_data = b"Secret data that needs protection";
        let password = "strong_password_123";
        
        let encryptor = FileEncryptor::from_password(password).unwrap();
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), test_data).unwrap();
        
        encryptor.encrypt_file(input_file.path(), encrypted_file.path())
            .expect("Encryption should succeed");
        
        encryptor.decrypt_file(encrypted_file.path(), decrypted_file.path())
            .expect("Decryption should succeed");
        
        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(test_data.to_vec(), decrypted_data);
    }
}use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use std::fs;
use std::io::{self, Write};
use std::path::Path;

pub struct FileEncryptor {
    cipher: Aes256Gcm,
}

impl FileEncryptor {
    pub fn new(key: &[u8; 32]) -> Self {
        let key = Key::<Aes256Gcm>::from_slice(key);
        let cipher = Aes256Gcm::new(key);
        FileEncryptor { cipher }
    }

    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> io::Result<()> {
        let data = fs::read(input_path)?;
        
        let mut nonce = [0u8; 12];
        OsRng.fill_bytes(&mut nonce);
        
        let encrypted_data = self.cipher
            .encrypt(Nonce::from_slice(&nonce), data.as_ref())
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        
        let mut output = fs::File::create(output_path)?;
        output.write_all(&nonce)?;
        output.write_all(&encrypted_data)?;
        
        Ok(())
    }

    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path) -> io::Result<()> {
        let data = fs::read(input_path)?;
        
        if data.len() < 12 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "File too short to contain nonce"
            ));
        }
        
        let nonce = &data[0..12];
        let encrypted_data = &data[12..];
        
        let decrypted_data = self.cipher
            .decrypt(Nonce::from_slice(nonce), encrypted_data)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        
        fs::write(output_path, decrypted_data)?;
        
        Ok(())
    }
}

pub fn generate_random_key() -> [u8; 32] {
    let mut key = [0u8; 32];
    OsRng.fill_bytes(&mut key);
    key
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let key = generate_random_key();
        let encryptor = FileEncryptor::new(&key);
        
        let test_data = b"Hello, this is a secret message!";
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), test_data).unwrap();
        
        encryptor.encrypt_file(input_file.path(), encrypted_file.path())
            .expect("Encryption failed");
        
        encryptor.decrypt_file(encrypted_file.path(), decrypted_file.path())
            .expect("Decryption failed");
        
        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(test_data.to_vec(), decrypted_data);
    }
}use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHash, PasswordHasher, PasswordVerifier, SaltString
    },
    Argon2
};
use std::fs;
use std::io::{Read, Write};
use std::path::Path;

const NONCE_SIZE: usize = 12;
const SALT_SIZE: usize = 16;

pub struct FileEncryptor {
    cipher: Aes256Gcm,
}

impl FileEncryptor {
    pub fn from_password(password: &str, salt: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        let argon2 = Argon2::default();
        let mut key = [0u8; 32];
        argon2.hash_password_into(password.as_bytes(), salt, &mut key)?;
        
        let key = Key::<Aes256Gcm>::from_slice(&key);
        let cipher = Aes256Gcm::new(key);
        
        Ok(FileEncryptor { cipher })
    }
    
    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let mut input_file = fs::File::open(input_path)?;
        let mut plaintext = Vec::new();
        input_file.read_to_end(&mut plaintext)?;
        
        let mut nonce = [0u8; NONCE_SIZE];
        OsRng.fill_bytes(&mut nonce);
        
        let ciphertext = self.cipher.encrypt(Nonce::from_slice(&nonce), plaintext.as_ref())
            .map_err(|e| format!("Encryption failed: {}", e))?;
        
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
            .map_err(|e| format!("Decryption failed: {}", e))?;
        
        let mut output_file = fs::File::create(output_path)?;
        output_file.write_all(&plaintext)?;
        
        Ok(())
    }
}

pub fn generate_salt() -> Vec<u8> {
    let mut salt = [0u8; SALT_SIZE];
    OsRng.fill_bytes(&mut salt);
    salt.to_vec()
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
        
        let original_content = b"Secret data that needs protection";
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), original_content).unwrap();
        
        encryptor.encrypt_file(input_file.path(), encrypted_file.path()).unwrap();
        encryptor.decrypt_file(encrypted_file.path(), decrypted_file.path()).unwrap();
        
        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(original_content.to_vec(), decrypted_content);
    }
    
    #[test]
    fn test_wrong_password_fails() {
        let password = "correct_password";
        let wrong_password = "wrong_password";
        let salt = generate_salt();
        
        let correct_encryptor = FileEncryptor::from_password(password, &salt).unwrap();
        let wrong_encryptor = FileEncryptor::from_password(wrong_password, &salt).unwrap();
        
        let original_content = b"Test data";
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), original_content).unwrap();
        
        correct_encryptor.encrypt_file(input_file.path(), encrypted_file.path()).unwrap();
        
        let result = wrong_encryptor.decrypt_file(encrypted_file.path(), decrypted_file.path());
        assert!(result.is_err());
    }
}
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use argon2::{
    password_hash::{rand_core::OsRng as ArgonOsRng, PasswordHasher, SaltString},
    Argon2,
};
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const NONCE_SIZE: usize = 12;
const SALT_SIZE: usize = 16;

pub struct EncryptionResult {
    pub ciphertext: Vec<u8>,
    pub nonce: [u8; NONCE_SIZE],
    pub salt: [u8; SALT_SIZE],
}

pub fn derive_key(password: &str, salt: &[u8]) -> Result<Key<Aes256Gcm>, String> {
    let salt_string = SaltString::encode_b64(salt).map_err(|e| e.to_string())?;
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt_string)
        .map_err(|e| e.to_string())?;

    let hash_bytes = password_hash.hash.ok_or("Hash generation failed")?;
    let key_bytes: [u8; 32] = hash_bytes.as_bytes()[..32]
        .try_into()
        .map_err(|_| "Hash too short")?;

    Ok(Key::<Aes256Gcm>::from_slice(&key_bytes).clone())
}

pub fn encrypt_data(data: &[u8], password: &str) -> Result<EncryptionResult, String> {
    let mut salt = [0u8; SALT_SIZE];
    ArgonOsRng.fill_bytes(&mut salt);

    let key = derive_key(password, &salt)?;
    let cipher = Aes256Gcm::new(&key);

    let mut nonce_bytes = [0u8; NONCE_SIZE];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, data)
        .map_err(|e| format!("Encryption failed: {}", e))?;

    Ok(EncryptionResult {
        ciphertext,
        nonce: nonce_bytes,
        salt,
    })
}

pub fn decrypt_data(
    encrypted: &EncryptionResult,
    password: &str,
) -> Result<Vec<u8>, String> {
    let key = derive_key(password, &encrypted.salt)?;
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(&encrypted.nonce);

    cipher
        .decrypt(nonce, encrypted.ciphertext.as_ref())
        .map_err(|e| format!("Decryption failed: {}", e))
}

pub fn encrypt_file(
    input_path: &Path,
    output_path: &Path,
    password: &str,
) -> Result<(), String> {
    let mut file = fs::File::open(input_path).map_err(|e| e.to_string())?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).map_err(|e| e.to_string())?;

    let encrypted = encrypt_data(&buffer, password)?;

    let mut output_file = fs::File::create(output_path).map_err(|e| e.to_string())?;
    output_file
        .write_all(&encrypted.salt)
        .map_err(|e| e.to_string())?;
    output_file
        .write_all(&encrypted.nonce)
        .map_err(|e| e.to_string())?;
    output_file
        .write_all(&encrypted.ciphertext)
        .map_err(|e| e.to_string())?;

    Ok(())
}

pub fn decrypt_file(
    input_path: &Path,
    output_path: &Path,
    password: &str,
) -> Result<(), String> {
    let mut file = fs::File::open(input_path).map_err(|e| e.to_string())?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).map_err(|e| e.to_string())?;

    if buffer.len() < SALT_SIZE + NONCE_SIZE {
        return Err("File too short to contain encrypted data".to_string());
    }

    let salt = buffer[..SALT_SIZE].try_into().unwrap();
    let nonce = buffer[SALT_SIZE..SALT_SIZE + NONCE_SIZE]
        .try_into()
        .unwrap();
    let ciphertext = buffer[SALT_SIZE + NONCE_SIZE..].to_vec();

    let encrypted = EncryptionResult {
        ciphertext,
        nonce,
        salt,
    };

    let decrypted = decrypt_data(&encrypted, password)?;

    let mut output_file = fs::File::create(output_path).map_err(|e| e.to_string())?;
    output_file
        .write_all(&decrypted)
        .map_err(|e| e.to_string())?;

    Ok(())
}

pub fn interactive_encrypt() -> Result<(), String> {
    println!("Enter input file path:");
    let mut input_path = String::new();
    io::stdin()
        .read_line(&mut input_path)
        .map_err(|e| e.to_string())?;
    let input_path = input_path.trim();

    println!("Enter output file path:");
    let mut output_path = String::new();
    io::stdin()
        .read_line(&mut output_path)
        .map_err(|e| e.to_string())?;
    let output_path = output_path.trim();

    println!("Enter password:");
    let mut password = String::new();
    io::stdin()
        .read_line(&mut password)
        .map_err(|e| e.to_string())?;
    let password = password.trim();

    encrypt_file(Path::new(input_path), Path::new(output_path), password)?;
    println!("Encryption completed successfully");
    Ok(())
}

pub fn interactive_decrypt() -> Result<(), String> {
    println!("Enter input file path:");
    let mut input_path = String::new();
    io::stdin()
        .read_line(&mut input_path)
        .map_err(|e| e.to_string())?;
    let input_path = input_path.trim();

    println!("Enter output file path:");
    let mut output_path = String::new();
    io::stdin()
        .read_line(&mut output_path)
        .map_err(|e| e.to_string())?;
    let output_path = output_path.trim();

    println!("Enter password:");
    let mut password = String::new();
    io::stdin()
        .read_line(&mut password)
        .map_err(|e| e.to_string())?;
    let password = password.trim();

    decrypt_file(Path::new(input_path), Path::new(output_path), password)?;
    println!("Decryption completed successfully");
    Ok(())
}