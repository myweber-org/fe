use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use argon2::{Argon2, PasswordHasher, PasswordVerifier};
use rand::RngCore;
use std::fs;
use std::io::{Read, Write};
use std::path::Path;

const SALT_LENGTH: usize = 16;
const NONCE_LENGTH: usize = 12;
const TAG_LENGTH: usize = 16;

pub struct EncryptionResult {
    pub ciphertext: Vec<u8>,
    pub salt: [u8; SALT_LENGTH],
    pub nonce: [u8; NONCE_LENGTH],
}

pub fn derive_key(password: &str, salt: &[u8]) -> Result<Key<Aes256Gcm>, String> {
    let argon2 = Argon2::default();
    let mut key = [0u8; 32];
    argon2
        .hash_password_into(password.as_bytes(), salt, &mut key)
        .map_err(|e| format!("Key derivation failed: {}", e))?;
    Ok(Key::<Aes256Gcm>::from_slice(&key).clone())
}

pub fn encrypt_data(data: &[u8], password: &str) -> Result<EncryptionResult, String> {
    let mut salt = [0u8; SALT_LENGTH];
    let mut nonce = [0u8; NONCE_LENGTH];
    OsRng.fill_bytes(&mut salt);
    OsRng.fill_bytes(&mut nonce);

    let key = derive_key(password, &salt)?;
    let cipher = Aes256Gcm::new(&key);
    let nonce_obj = Nonce::from_slice(&nonce);

    let ciphertext = cipher
        .encrypt(nonce_obj, data)
        .map_err(|e| format!("Encryption failed: {}", e))?;

    Ok(EncryptionResult {
        ciphertext,
        salt,
        nonce,
    })
}

pub fn decrypt_data(
    encrypted: &EncryptionResult,
    password: &str,
) -> Result<Vec<u8>, String> {
    let key = derive_key(password, &encrypted.salt)?;
    let cipher = Aes256Gcm::new(&key);
    let nonce_obj = Nonce::from_slice(&encrypted.nonce);

    cipher
        .decrypt(nonce_obj, encrypted.ciphertext.as_ref())
        .map_err(|e| format!("Decryption failed: {}", e))
}

pub fn encrypt_file(
    input_path: &Path,
    output_path: &Path,
    password: &str,
) -> Result<(), String> {
    let mut file = fs::File::open(input_path)
        .map_err(|e| format!("Failed to open input file: {}", e))?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)
        .map_err(|e| format!("Failed to read file: {}", e))?;

    let encrypted = encrypt_data(&buffer, password)?;

    let mut output = fs::File::create(output_path)
        .map_err(|e| format!("Failed to create output file: {}", e))?;
    output
        .write_all(&encrypted.salt)
        .map_err(|e| format!("Failed to write salt: {}", e))?;
    output
        .write_all(&encrypted.nonce)
        .map_err(|e| format!("Failed to write nonce: {}", e))?;
    output
        .write_all(&encrypted.ciphertext)
        .map_err(|e| format!("Failed to write ciphertext: {}", e))?;

    Ok(())
}

pub fn decrypt_file(
    input_path: &Path,
    output_path: &Path,
    password: &str,
) -> Result<(), String> {
    let mut file = fs::File::open(input_path)
        .map_err(|e| format!("Failed to open encrypted file: {}", e))?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)
        .map_err(|e| format!("Failed to read encrypted file: {}", e))?;

    if buffer.len() < SALT_LENGTH + NONCE_LENGTH {
        return Err("File too short to contain valid encrypted data".to_string());
    }

    let salt = buffer[..SALT_LENGTH].try_into().unwrap();
    let nonce = buffer[SALT_LENGTH..SALT_LENGTH + NONCE_LENGTH]
        .try_into()
        .unwrap();
    let ciphertext = buffer[SALT_LENGTH + NONCE_LENGTH..].to_vec();

    let encrypted = EncryptionResult {
        ciphertext,
        salt,
        nonce,
    };

    let decrypted = decrypt_data(&encrypted, password)?;

    let mut output = fs::File::create(output_path)
        .map_err(|e| format!("Failed to create output file: {}", e))?;
    output
        .write_all(&decrypted)
        .map_err(|e| format!("Failed to write decrypted data: {}", e))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_roundtrip() {
        let data = b"Secret data that needs protection";
        let password = "strong_password_123";

        let encrypted = encrypt_data(data, password).unwrap();
        let decrypted = decrypt_data(&encrypted, password).unwrap();

        assert_eq!(data.to_vec(), decrypted);
    }

    #[test]
    fn test_wrong_password_fails() {
        let data = b"Test data";
        let password = "correct_password";
        let wrong_password = "wrong_password";

        let encrypted = encrypt_data(data, password).unwrap();
        let result = decrypt_data(&encrypted, wrong_password);

        assert!(result.is_err());
    }

    #[test]
    fn test_file_encryption_roundtrip() {
        let temp_input = NamedTempFile::new().unwrap();
        let temp_encrypted = NamedTempFile::new().unwrap();
        let temp_decrypted = NamedTempFile::new().unwrap();

        let original_data = b"File content to encrypt";
        fs::write(temp_input.path(), original_data).unwrap();

        let password = "file_encryption_password";

        encrypt_file(temp_input.path(), temp_encrypted.path(), password).unwrap();
        decrypt_file(temp_encrypted.path(), temp_decrypted.path(), password).unwrap();

        let decrypted_data = fs::read(temp_decrypted.path()).unwrap();
        assert_eq!(original_data.to_vec(), decrypted_data);
    }
}use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use hex;
use rand::RngCore;
use std::error::Error;

type Aes256CbcEnc = cbc::Encryptor<aes::Aes256>;
type Aes256CbcDec = cbc::Decryptor<aes::Aes256>;

pub struct EncryptionResult {
    pub ciphertext: String,
    pub iv: String,
}

pub fn encrypt_data(plaintext: &str, key: &[u8; 32]) -> Result<EncryptionResult, Box<dyn Error>> {
    if key.len() != 32 {
        return Err("Key must be 32 bytes for AES-256".into());
    }

    let mut iv = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut iv);

    let ciphertext = Aes256CbcEnc::new(key.into(), &iv.into())
        .encrypt_padded_vec_mut::<Pkcs7>(plaintext.as_bytes());

    Ok(EncryptionResult {
        ciphertext: hex::encode(ciphertext),
        iv: hex::encode(iv),
    })
}

pub fn decrypt_data(ciphertext_hex: &str, iv_hex: &str, key: &[u8; 32]) -> Result<String, Box<dyn Error>> {
    let ciphertext = hex::decode(ciphertext_hex)?;
    let iv = hex::decode(iv_hex)?;

    if iv.len() != 16 {
        return Err("IV must be 16 bytes".into());
    }

    let decrypted = Aes256CbcDec::new(key.into(), &iv.into())
        .decrypt_padded_vec_mut::<Pkcs7>(&ciphertext)
        .map_err(|e| format!("Decryption failed: {}", e))?;

    String::from_utf8(decrypted).map_err(|e| e.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_roundtrip() {
        let key = [0x42u8; 32];
        let plaintext = "Sensitive data requiring protection";

        let encrypted = encrypt_data(plaintext, &key).unwrap();
        let decrypted = decrypt_data(&encrypted.ciphertext, &encrypted.iv, &key).unwrap();

        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_invalid_key_length() {
        let short_key = [0x42u8; 16];
        let plaintext = "test";

        let result = encrypt_data(plaintext, &short_key);
        assert!(result.is_err());
    }
}
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use std::fs;

pub fn encrypt_file(input_path: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let data = fs::read(input_path)?;
    
    let key = Aes256Gcm::generate_key(&mut OsRng);
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(b"unique nonce");
    
    let encrypted_data = cipher.encrypt(nonce, data.as_ref())
        .map_err(|e| format!("Encryption failed: {}", e))?;
    
    fs::write(output_path, encrypted_data)?;
    
    let key_path = format!("{}.key", output_path);
    fs::write(key_path, key.as_slice())?;
    
    Ok(())
}

pub fn decrypt_file(input_path: &str, key_path: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let encrypted_data = fs::read(input_path)?;
    let key_bytes = fs::read(key_path)?;
    
    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(b"unique nonce");
    
    let decrypted_data = cipher.decrypt(nonce, encrypted_data.as_ref())
        .map_err(|e| format!("Decryption failed: {}", e))?;
    
    fs::write(output_path, decrypted_data)?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_roundtrip() {
        let mut plaintext_file = NamedTempFile::new().unwrap();
        writeln!(plaintext_file, "Sensitive data: {}", "Test content").unwrap();
        
        let encrypted_file = NamedTempFile::new().unwrap();
        let key_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        encrypt_file(
            plaintext_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap()
        ).unwrap();
        
        let key_path = format!("{}.key", encrypted_file.path().to_str().unwrap());
        fs::copy(&key_path, key_file.path()).unwrap();
        
        decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            key_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap()
        ).unwrap();
        
        let original = fs::read_to_string(plaintext_file.path()).unwrap();
        let decrypted = fs::read_to_string(decrypted_file.path()).unwrap();
        
        assert_eq!(original, decrypted);
        fs::remove_file(key_path).ok();
    }
}