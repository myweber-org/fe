use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use argon2::{
    password_hash::{rand_core::OsRng as ArgonRng, PasswordHasher, SaltString},
    Argon2, ParamsBuilder,
};
use std::{
    fs::{self, File},
    io::{Read, Write},
    path::Path,
};

const SALT_LENGTH: usize = 16;
const NONCE_LENGTH: usize = 12;

pub struct EncryptionResult {
    pub encrypted_data: Vec<u8>,
    pub salt: [u8; SALT_LENGTH],
    pub nonce: [u8; NONCE_LENGTH],
}

pub fn derive_key(password: &str, salt: &[u8]) -> Result<Key<Aes256Gcm>, String> {
    let salt_str = SaltString::encode_b64(salt).map_err(|e| e.to_string())?;
    
    let argon2 = Argon2::new(
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        ParamsBuilder::new()
            .m_cost(65536)
            .t_cost(3)
            .p_cost(4)
            .build()
            .map_err(|e| e.to_string())?,
    );

    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt_str)
        .map_err(|e| e.to_string())?;

    let hash_bytes = password_hash.hash.ok_or("Hash generation failed")?;
    let key_bytes: [u8; 32] = hash_bytes.as_bytes()[..32]
        .try_into()
        .map_err(|_| "Key derivation failed")?;

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

    let mut salt = [0u8; SALT_LENGTH];
    ArgonRng.fill_bytes(&mut salt);

    let key = derive_key(password, &salt)?;

    let cipher = Aes256Gcm::new(&key);
    let mut nonce_bytes = [0u8; NONCE_LENGTH];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let encrypted_data = cipher
        .encrypt(nonce, plaintext.as_ref())
        .map_err(|e| e.to_string())?;

    let mut output_file = File::create(output_path).map_err(|e| e.to_string())?;
    output_file
        .write_all(&encrypted_data)
        .map_err(|e| e.to_string())?;

    Ok(EncryptionResult {
        encrypted_data,
        salt,
        nonce: nonce_bytes,
    })
}

pub fn decrypt_file(
    input_path: &Path,
    output_path: &Path,
    password: &str,
    salt: &[u8; SALT_LENGTH],
    nonce: &[u8; NONCE_LENGTH],
) -> Result<Vec<u8>, String> {
    let mut file = File::open(input_path).map_err(|e| e.to_string())?;
    let mut ciphertext = Vec::new();
    file.read_to_end(&mut ciphertext).map_err(|e| e.to_string())?;

    let key = derive_key(password, salt)?;
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(nonce);

    let decrypted_data = cipher
        .decrypt(nonce, ciphertext.as_ref())
        .map_err(|e| e.to_string())?;

    fs::write(output_path, &decrypted_data).map_err(|e| e.to_string())?;

    Ok(decrypted_data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let plaintext = b"Test data for encryption and decryption";
        let password = "secure_password_123";

        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        fs::write(input_file.path(), plaintext).unwrap();

        let enc_result = encrypt_file(input_file.path(), encrypted_file.path(), password).unwrap();

        let decrypted = decrypt_file(
            encrypted_file.path(),
            decrypted_file.path(),
            password,
            &enc_result.salt,
            &enc_result.nonce,
        ).unwrap();

        assert_eq!(plaintext.to_vec(), decrypted);
    }

    #[test]
    fn test_wrong_password_fails() {
        let plaintext = b"Sensitive data";
        let correct_password = "correct_pass";
        let wrong_password = "wrong_pass";

        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        fs::write(input_file.path(), plaintext).unwrap();

        let enc_result = encrypt_file(input_file.path(), encrypted_file.path(), correct_password).unwrap();

        let result = decrypt_file(
            encrypted_file.path(),
            decrypted_file.path(),
            wrong_password,
            &enc_result.salt,
            &enc_result.nonce,
        );

        assert!(result.is_err());
    }
}use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: u8 = 0x55;

pub fn encrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    let encryption_key = key.unwrap_or(DEFAULT_KEY);
    let data = fs::read(input_path)?;
    
    let encrypted_data: Vec<u8> = data
        .iter()
        .map(|byte| byte ^ encryption_key)
        .collect();
    
    fs::write(output_path, encrypted_data)?;
    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    encrypt_file(input_path, output_path, key)
}

pub fn process_stream<R: Read, W: Write>(mut reader: R, mut writer: W, key: u8) -> io::Result<()> {
    let mut buffer = [0; 1024];
    
    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        
        for byte in buffer[..bytes_read].iter_mut() {
            *byte ^= key;
        }
        
        writer.write_all(&buffer[..bytes_read])?;
    }
    
    writer.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    
    #[test]
    fn test_xor_encryption() {
        let data = b"Hello, World!";
        let key = 0x42;
        
        let encrypted: Vec<u8> = data.iter().map(|b| b ^ key).collect();
        let decrypted: Vec<u8> = encrypted.iter().map(|b| b ^ key).collect();
        
        assert_eq!(data.to_vec(), decrypted);
    }
    
    #[test]
    fn test_stream_processing() {
        let input = b"Test data stream";
        let key = 0x77;
        
        let mut reader = Cursor::new(input);
        let mut writer = Cursor::new(Vec::new());
        
        process_stream(&mut reader, &mut writer, key).unwrap();
        
        let encrypted = writer.into_inner();
        let mut reader2 = Cursor::new(&encrypted);
        let mut writer2 = Cursor::new(Vec::new());
        
        process_stream(&mut reader2, &mut writer2, key).unwrap();
        
        assert_eq!(input, writer2.into_inner().as_slice());
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
    cipher: Aes256Gcm,
}

impl FileEncryptor {
    pub fn new(password: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let salt = SaltString::generate(&mut ArgonRng);
        let argon2 = Argon2::default();
        
        let password_hash = argon2.hash_password(password.as_bytes(), &salt)?;
        let key_bytes = password_hash.hash.ok_or("Hash generation failed")?;
        
        let key = Key::<Aes256Gcm>::from_slice(key_bytes.as_bytes());
        let cipher = Aes256Gcm::new(key);
        
        Ok(Self { cipher })
    }
    
    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let mut input_file = fs::File::open(input_path)?;
        let mut plaintext = Vec::new();
        input_file.read_to_end(&mut plaintext)?;
        
        let mut nonce_bytes = [0u8; NONCE_SIZE];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        let ciphertext = self.cipher.encrypt(nonce, plaintext.as_ref())
            .map_err(|e| format!("Encryption failed: {}", e))?;
        
        let mut output_file = fs::File::create(output_path)?;
        output_file.write_all(&nonce_bytes)?;
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

pub fn process_encryption() -> Result<(), Box<dyn std::error::Error>> {
    println!("Enter password for encryption: ");
    io::Write::flush(&mut io::stdout())?;
    
    let mut password = String::new();
    io::stdin().read_line(&mut password)?;
    let password = password.trim();
    
    let encryptor = FileEncryptor::new(password)?;
    
    let test_data = b"Sample confidential data for encryption demonstration";
    let input_path = Path::new("test_input.bin");
    let encrypted_path = Path::new("test_encrypted.bin");
    let decrypted_path = Path::new("test_decrypted.bin");
    
    fs::write(input_path, test_data)?;
    
    encryptor.encrypt_file(input_path, encrypted_path)?;
    println!("File encrypted successfully");
    
    encryptor.decrypt_file(encrypted_path, decrypted_path)?;
    println!("File decrypted successfully");
    
    let decrypted_data = fs::read(decrypted_path)?;
    assert_eq!(test_data.as_ref(), decrypted_data.as_slice());
    println!("Encryption/decryption verification passed");
    
    fs::remove_file(input_path).ok();
    fs::remove_file(encrypted_path).ok();
    fs::remove_file(decrypted_path).ok();
    
    Ok(())
}

fn main() {
    if let Err(e) = process_encryption() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use std::fs;
use std::io::{Read, Write};
use std::path::Path;

pub struct FileEncryptor {
    cipher: Aes256Gcm,
}

impl FileEncryptor {
    pub fn new() -> Self {
        let key = Aes256Gcm::generate_key(&mut OsRng);
        let cipher = Aes256Gcm::new(&key);
        FileEncryptor { cipher }
    }

    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), String> {
        let mut file = fs::File::open(input_path)
            .map_err(|e| format!("Failed to open input file: {}", e))?;
        
        let mut plaintext = Vec::new();
        file.read_to_end(&mut plaintext)
            .map_err(|e| format!("Failed to read file: {}", e))?;

        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let ciphertext = self.cipher
            .encrypt(&nonce, plaintext.as_ref())
            .map_err(|e| format!("Encryption failed: {}", e))?;

        let mut output = fs::File::create(output_path)
            .map_err(|e| format!("Failed to create output file: {}", e))?;

        output.write_all(&nonce)
            .map_err(|e| format!("Failed to write nonce: {}", e))?;
        output.write_all(&ciphertext)
            .map_err(|e| format!("Failed to write ciphertext: {}", e))?;

        Ok(())
    }

    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), String> {
        let mut file = fs::File::open(input_path)
            .map_err(|e| format!("Failed to open encrypted file: {}", e))?;
        
        let mut data = Vec::new();
        file.read_to_end(&mut data)
            .map_err(|e| format!("Failed to read encrypted file: {}", e))?;

        if data.len() < 12 {
            return Err("Invalid encrypted file format".to_string());
        }

        let (nonce_bytes, ciphertext) = data.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);
        
        let plaintext = self.cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| format!("Decryption failed: {}", e))?;

        fs::write(output_path, plaintext)
            .map_err(|e| format!("Failed to write decrypted file: {}", e))?;

        Ok(())
    }
}

pub fn generate_secure_key() -> Vec<u8> {
    let key = Aes256Gcm::generate_key(&mut OsRng);
    key.to_vec()
}use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use chacha20poly1305::{ChaCha20Poly1305, Key as ChaChaKey, Nonce as ChaChaNonce};
use rand::RngCore;
use std::error::Error;

const AES_NONCE_SIZE: usize = 12;
const CHACHA_NONCE_SIZE: usize = 12;

pub enum CipherType {
    Aes256Gcm,
    ChaCha20Poly1305,
}

pub struct EncryptionResult {
    pub ciphertext: Vec<u8>,
    pub nonce: Vec<u8>,
}

pub fn encrypt_data(
    plaintext: &[u8],
    key: &[u8],
    cipher_type: CipherType,
) -> Result<EncryptionResult, Box<dyn Error>> {
    match cipher_type {
        CipherType::Aes256Gcm => {
            let cipher = Aes256Gcm::new_from_slice(key)?;
            let mut nonce_bytes = [0u8; AES_NONCE_SIZE];
            OsRng.fill_bytes(&mut nonce_bytes);
            let nonce = Nonce::from_slice(&nonce_bytes);
            
            let ciphertext = cipher.encrypt(nonce, plaintext)?;
            
            Ok(EncryptionResult {
                ciphertext,
                nonce: nonce_bytes.to_vec(),
            })
        }
        CipherType::ChaCha20Poly1305 => {
            let cipher = ChaCha20Poly1305::new_from_slice(key)?;
            let mut nonce_bytes = [0u8; CHACHA_NONCE_SIZE];
            OsRng.fill_bytes(&mut nonce_bytes);
            let nonce = ChaChaNonce::from_slice(&nonce_bytes);
            
            let ciphertext = cipher.encrypt(nonce, plaintext)?;
            
            Ok(EncryptionResult {
                ciphertext,
                nonce: nonce_bytes.to_vec(),
            })
        }
    }
}

pub fn decrypt_data(
    ciphertext: &[u8],
    key: &[u8],
    nonce: &[u8],
    cipher_type: CipherType,
) -> Result<Vec<u8>, Box<dyn Error>> {
    match cipher_type {
        CipherType::Aes256Gcm => {
            let cipher = Aes256Gcm::new_from_slice(key)?;
            let nonce = Nonce::from_slice(nonce);
            cipher.decrypt(nonce, ciphertext).map_err(|e| e.into())
        }
        CipherType::ChaCha20Poly1305 => {
            let cipher = ChaCha20Poly1305::new_from_slice(key)?;
            let nonce = ChaChaNonce::from_slice(nonce);
            cipher.decrypt(nonce, ciphertext).map_err(|e| e.into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hex::encode;

    #[test]
    fn test_aes_encryption_decryption() {
        let key = b"0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
        let plaintext = b"Test encryption data";
        
        let result = encrypt_data(plaintext, key, CipherType::Aes256Gcm).unwrap();
        let decrypted = decrypt_data(&result.ciphertext, key, &result.nonce, CipherType::Aes256Gcm).unwrap();
        
        assert_eq!(plaintext.to_vec(), decrypted);
    }

    #[test]
    fn test_chacha_encryption_decryption() {
        let key = b"0123456789abcdef0123456789abcdef0123456789abcdef";
        let plaintext = b"Another test message";
        
        let result = encrypt_data(plaintext, key, CipherType::ChaCha20Poly1305).unwrap();
        let decrypted = decrypt_data(&result.ciphertext, key, &result.nonce, CipherType::ChaCha20Poly1305).unwrap();
        
        assert_eq!(plaintext.to_vec(), decrypted);
    }
}