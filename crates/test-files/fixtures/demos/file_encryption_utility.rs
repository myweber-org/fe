use aes_gcm::{
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
use std::{
    fs::{self, File},
    io::{Read, Write},
    path::Path
};

const NONCE_SIZE: usize = 12;
const SALT_SIZE: usize = 16;

pub struct FileEncryptor {
    cipher: Aes256Gcm,
    nonce: [u8; NONCE_SIZE],
}

impl FileEncryptor {
    pub fn from_password(password: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2.hash_password(password.as_bytes(), &salt)?;
        
        let key_bytes = password_hash.hash.ok_or("Hash generation failed")?;
        let key = Key::<Aes256Gcm>::from_slice(key_bytes.as_bytes());
        let cipher = Aes256Gcm::new(key);
        
        let mut nonce = [0u8; NONCE_SIZE];
        OsRng.fill_bytes(&mut nonce);
        
        Ok(Self {
            cipher,
            nonce,
        })
    }
    
    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let mut input_file = File::open(input_path)?;
        let mut plaintext = Vec::new();
        input_file.read_to_end(&mut plaintext)?;
        
        let ciphertext = self.cipher.encrypt(Nonce::from_slice(&self.nonce), plaintext.as_ref())
            .map_err(|e| format!("Encryption failed: {}", e))?;
        
        let mut output_file = File::create(output_path)?;
        output_file.write_all(&self.nonce)?;
        output_file.write_all(&ciphertext)?;
        
        Ok(())
    }
    
    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let mut input_file = File::open(input_path)?;
        let mut encrypted_data = Vec::new();
        input_file.read_to_end(&mut encrypted_data)?;
        
        if encrypted_data.len() < NONCE_SIZE {
            return Err("Invalid encrypted file format".into());
        }
        
        let (nonce_bytes, ciphertext) = encrypted_data.split_at(NONCE_SIZE);
        let nonce = Nonce::from_slice(nonce_bytes);
        
        let plaintext = self.cipher.decrypt(nonce, ciphertext)
            .map_err(|e| format!("Decryption failed: {}", e))?;
        
        let mut output_file = File::create(output_path)?;
        output_file.write_all(&plaintext)?;
        
        Ok(())
    }
}

pub fn encrypt_directory(password: &str, dir_path: &Path, output_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    if !dir_path.is_dir() {
        return Err("Provided path is not a directory".into());
    }
    
    fs::create_dir_all(output_dir)?;
    let encryptor = FileEncryptor::from_password(password)?;
    
    for entry in fs::read_dir(dir_path)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_file() {
            let output_path = output_dir.join(path.file_name().ok_or("Invalid filename")?);
            encryptor.encrypt_file(&path, &output_path)?;
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[test]
    fn test_file_encryption_decryption() {
        let temp_dir = tempdir().unwrap();
        let original_path = temp_dir.path().join("test.txt");
        let encrypted_path = temp_dir.path().join("test.enc");
        let decrypted_path = temp_dir.path().join("test_decrypted.txt");
        
        let test_content = b"Secret data that needs protection";
        fs::write(&original_path, test_content).unwrap();
        
        let encryptor = FileEncryptor::from_password("strong_password").unwrap();
        encryptor.encrypt_file(&original_path, &encrypted_path).unwrap();
        encryptor.decrypt_file(&encrypted_path, &decrypted_path).unwrap();
        
        let decrypted_content = fs::read(&decrypted_path).unwrap();
        assert_eq!(test_content.to_vec(), decrypted_content);
    }
    
    #[test]
    fn test_directory_encryption() {
        let source_dir = tempdir().unwrap();
        let output_dir = tempdir().unwrap();
        
        let file1_path = source_dir.path().join("file1.txt");
        let file2_path = source_dir.path().join("file2.txt");
        
        fs::write(&file1_path, b"First secret file").unwrap();
        fs::write(&file2_path, b"Second secret file").unwrap();
        
        encrypt_directory("directory_password", source_dir.path(), output_dir.path()).unwrap();
        
        let encrypted_files: Vec<_> = fs::read_dir(output_dir.path()).unwrap()
            .map(|entry| entry.unwrap().path())
            .collect();
        
        assert_eq!(encrypted_files.len(), 2);
    }
}use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: u8 = 0x55;

pub fn encrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    let encryption_key = key.unwrap_or(DEFAULT_KEY);
    let mut input_file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    input_file.read_to_end(&mut buffer)?;

    for byte in &mut buffer {
        *byte ^= encryption_key;
    }

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&buffer)?;
    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    encrypt_file(input_path, output_path, key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_encryption_decryption() {
        let test_data = b"Hello, World!";
        let input_file = "test_input.txt";
        let encrypted_file = "test_encrypted.bin";
        let decrypted_file = "test_decrypted.txt";

        fs::write(input_file, test_data).unwrap();

        encrypt_file(input_file, encrypted_file, Some(0x42)).unwrap();
        decrypt_file(encrypted_file, decrypted_file, Some(0x42)).unwrap();

        let decrypted_content = fs::read(decrypted_file).unwrap();
        assert_eq!(decrypted_content, test_data);

        fs::remove_file(input_file).ok();
        fs::remove_file(encrypted_file).ok();
        fs::remove_file(decrypted_file).ok();
    }

    #[test]
    fn test_default_key() {
        let test_data = b"Test data";
        let input_file = "test_default_input.txt";
        let encrypted_file = "test_default_encrypted.bin";
        let decrypted_file = "test_default_decrypted.txt";

        fs::write(input_file, test_data).unwrap();

        encrypt_file(input_file, encrypted_file, None).unwrap();
        decrypt_file(encrypted_file, decrypted_file, None).unwrap();

        let decrypted_content = fs::read(decrypted_file).unwrap();
        assert_eq!(decrypted_content, test_data);

        fs::remove_file(input_file).ok();
        fs::remove_file(encrypted_file).ok();
        fs::remove_file(decrypted_file).ok();
    }
}
use aes_gcm::{
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
use std::io::{self, Read, Write};

const NONCE_SIZE: usize = 12;
const SALT_SIZE: usize = 16;

pub struct EncryptionResult {
    pub ciphertext: Vec<u8>,
    pub nonce: [u8; NONCE_SIZE],
    pub salt: [u8; SALT_SIZE],
}

pub fn derive_key(password: &str, salt: &[u8]) -> Result<Key<Aes256Gcm>, String> {
    let salt_str = SaltString::encode_b64(salt)
        .map_err(|e| format!("Salt encoding failed: {}", e))?;
    
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt_str)
        .map_err(|e| format!("Key derivation failed: {}", e))?;
    
    let hash_bytes = password_hash.hash.ok_or("No hash generated")?.as_bytes();
    if hash_bytes.len() < 32 {
        return Err("Derived key too short".to_string());
    }
    
    let key_slice = &hash_bytes[..32];
    Ok(*Key::<Aes256Gcm>::from_slice(key_slice))
}

pub fn encrypt_file(
    input_path: &str,
    output_path: &str,
    password: &str
) -> Result<EncryptionResult, String> {
    let mut file_data = fs::read(input_path)
        .map_err(|e| format!("Failed to read input file: {}", e))?;
    
    let mut salt = [0u8; SALT_SIZE];
    OsRng.fill_bytes(&mut salt);
    
    let key = derive_key(password, &salt)?;
    
    let cipher = Aes256Gcm::new(&key);
    
    let mut nonce_bytes = [0u8; NONCE_SIZE];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    
    let ciphertext = cipher
        .encrypt(nonce, file_data.as_ref())
        .map_err(|e| format!("Encryption failed: {}", e))?;
    
    let mut output_file = fs::File::create(output_path)
        .map_err(|e| format!("Failed to create output file: {}", e))?;
    
    output_file.write_all(&salt)
        .and_then(|_| output_file.write_all(&nonce_bytes))
        .and_then(|_| output_file.write_all(&ciphertext))
        .map_err(|e| format!("Failed to write encrypted data: {}", e))?;
    
    Ok(EncryptionResult {
        ciphertext,
        nonce: nonce_bytes,
        salt,
    })
}

pub fn decrypt_file(
    input_path: &str,
    output_path: &str,
    password: &str
) -> Result<Vec<u8>, String> {
    let mut input_file = fs::File::open(input_path)
        .map_err(|e| format!("Failed to open encrypted file: {}", e))?;
    
    let mut salt = [0u8; SALT_SIZE];
    input_file.read_exact(&mut salt)
        .map_err(|e| format!("Failed to read salt: {}", e))?;
    
    let mut nonce_bytes = [0u8; NONCE_SIZE];
    input_file.read_exact(&mut nonce_bytes)
        .map_err(|e| format!("Failed to read nonce: {}", e))?;
    
    let mut ciphertext = Vec::new();
    input_file.read_to_end(&mut ciphertext)
        .map_err(|e| format!("Failed to read ciphertext: {}", e))?;
    
    let key = derive_key(password, &salt)?;
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(&nonce_bytes);
    
    let plaintext = cipher
        .decrypt(nonce, ciphertext.as_ref())
        .map_err(|e| format!("Decryption failed: {}", e))?;
    
    fs::write(output_path, &plaintext)
        .map_err(|e| format!("Failed to write decrypted file: {}", e))?;
    
    Ok(plaintext)
}

pub fn encrypt_data(data: &[u8], password: &str) -> Result<EncryptionResult, String> {
    let mut salt = [0u8; SALT_SIZE];
    OsRng.fill_bytes(&mut salt);
    
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
    ciphertext: &[u8],
    nonce: &[u8; NONCE_SIZE],
    salt: &[u8; SALT_SIZE],
    password: &str
) -> Result<Vec<u8>, String> {
    let key = derive_key(password, salt)?;
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(nonce);
    
    cipher.decrypt(nonce, ciphertext)
        .map_err(|e| format!("Decryption failed: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encrypt_decrypt_data() {
        let test_data = b"Test encryption and decryption";
        let password = "secure_password_123";
        
        let encrypted = encrypt_data(test_data, password).unwrap();
        let decrypted = decrypt_data(
            &encrypted.ciphertext,
            &encrypted.nonce,
            &encrypted.salt,
            password
        ).unwrap();
        
        assert_eq!(test_data.to_vec(), decrypted);
    }

    #[test]
    fn test_encrypt_decrypt_file() {
        let test_content = b"File encryption test content";
        let password = "another_secure_password";
        
        let input_file = NamedTempFile::new().unwrap();
        let output_encrypted = NamedTempFile::new().unwrap();
        let output_decrypted = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), test_content).unwrap();
        
        encrypt_file(
            input_file.path().to_str().unwrap(),
            output_encrypted.path().to_str().unwrap(),
            password
        ).unwrap();
        
        decrypt_file(
            output_encrypted.path().to_str().unwrap(),
            output_decrypted.path().to_str().unwrap(),
            password
        ).unwrap();
        
        let decrypted_content = fs::read(output_decrypted.path()).unwrap();
        assert_eq!(test_content.to_vec(), decrypted_content);
    }

    #[test]
    fn test_wrong_password_fails() {
        let test_data = b"Sensitive data";
        let password = "correct_password";
        let wrong_password = "wrong_password";
        
        let encrypted = encrypt_data(test_data, password).unwrap();
        
        let result = decrypt_data(
            &encrypted.ciphertext,
            &encrypted.nonce,
            &encrypted.salt,
            wrong_password
        );
        
        assert!(result.is_err());
    }
}