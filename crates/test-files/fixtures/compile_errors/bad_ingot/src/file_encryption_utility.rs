
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
use std::io::{self, Write};

const NONCE_SIZE: usize = 12;
const SALT_SIZE: usize = 16;

pub struct EncryptionResult {
    pub ciphertext: Vec<u8>,
    pub nonce: [u8; NONCE_SIZE],
    pub salt: [u8; SALT_SIZE],
}

pub fn encrypt_file(
    file_path: &str,
    password: &str,
) -> io::Result<EncryptionResult> {
    let plaintext = fs::read(file_path)?;
    
    let mut salt = [0u8; SALT_SIZE];
    OsRng.fill_bytes(&mut salt);
    
    let argon2 = Argon2::default();
    let salt_string = SaltString::encode_b64(&salt)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt_string)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    
    let key_bytes = password_hash.hash.unwrap().as_bytes();
    let key = Key::<Aes256Gcm>::from_slice(&key_bytes[..32]);
    let cipher = Aes256Gcm::new(key);
    
    let mut nonce_bytes = [0u8; NONCE_SIZE];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    
    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_ref())
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    
    Ok(EncryptionResult {
        ciphertext,
        nonce: nonce_bytes,
        salt,
    })
}

pub fn decrypt_file(
    encrypted_data: &EncryptionResult,
    password: &str,
) -> io::Result<Vec<u8>> {
    let argon2 = Argon2::default();
    let salt_string = SaltString::encode_b64(&encrypted_data.salt)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt_string)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    
    let key_bytes = password_hash.hash.unwrap().as_bytes();
    let key = Key::<Aes256Gcm>::from_slice(&key_bytes[..32]);
    let cipher = Aes256Gcm::new(key);
    
    let nonce = Nonce::from_slice(&encrypted_data.nonce);
    
    cipher
        .decrypt(nonce, encrypted_data.ciphertext.as_ref())
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
}

pub fn save_encrypted_data(
    result: &EncryptionResult,
    output_path: &str,
) -> io::Result<()> {
    let mut file = fs::File::create(output_path)?;
    
    file.write_all(&result.salt)?;
    file.write_all(&result.nonce)?;
    file.write_all(&result.ciphertext)?;
    
    Ok(())
}

pub fn load_encrypted_data(
    input_path: &str,
) -> io::Result<EncryptionResult> {
    let data = fs::read(input_path)?;
    
    if data.len() < SALT_SIZE + NONCE_SIZE {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "File too short to contain encrypted data"
        ));
    }
    
    let salt = data[..SALT_SIZE].try_into()
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid salt"))?;
    
    let nonce = data[SALT_SIZE..SALT_SIZE + NONCE_SIZE].try_into()
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid nonce"))?;
    
    let ciphertext = data[SALT_SIZE + NONCE_SIZE..].to_vec();
    
    Ok(EncryptionResult {
        ciphertext,
        nonce,
        salt,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_encryption_decryption() {
        let test_content = b"Test secret data for encryption";
        let password = "strong_password_123";
        
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(test_content).unwrap();
        
        let encrypted = encrypt_file(
            temp_file.path().to_str().unwrap(),
            password
        ).unwrap();
        
        let decrypted = decrypt_file(&encrypted, password).unwrap();
        
        assert_eq!(decrypted, test_content);
    }
    
    #[test]
    fn test_wrong_password_fails() {
        let test_content = b"Secret data";
        let correct_password = "correct_password";
        let wrong_password = "wrong_password";
        
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(test_content).unwrap();
        
        let encrypted = encrypt_file(
            temp_file.path().to_str().unwrap(),
            correct_password
        ).unwrap();
        
        let result = decrypt_file(&encrypted, wrong_password);
        assert!(result.is_err());
    }
}use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use argon2::{
    password_hash::{rand_core::OsRng as ArgonRng, PasswordHasher, SaltString},
    Argon2, Params,
};
use std::{
    fs::{self, File},
    io::{Read, Write},
    path::Path,
};

const NONCE_SIZE: usize = 12;
const SALT_SIZE: usize = 16;

pub struct EncryptionResult {
    pub encrypted_data: Vec<u8>,
    pub nonce: [u8; NONCE_SIZE],
    pub salt: [u8; SALT_SIZE],
}

pub fn derive_key(password: &str, salt: &[u8]) -> Result<Key<Aes256Gcm>, String> {
    let argon2 = Argon2::new(
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        Params::new(15000, 2, 1, Some(32)).map_err(|e| e.to_string())?,
    );

    let salt_str = SaltString::encode_b64(salt).map_err(|e| e.to_string())?;
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt_str)
        .map_err(|e| e.to_string())?;

    let key_bytes = password_hash.hash.ok_or("Hash generation failed")?;
    let key_slice: &[u8] = key_bytes.as_bytes();
    
    if key_slice.len() != 32 {
        return Err("Invalid key length".to_string());
    }

    Ok(*Key::<Aes256Gcm>::from_slice(key_slice))
}

pub fn encrypt_file(
    input_path: &Path,
    output_path: &Path,
    password: &str,
) -> Result<EncryptionResult, String> {
    let mut file = File::open(input_path).map_err(|e| e.to_string())?;
    let mut plaintext = Vec::new();
    file.read_to_end(&mut plaintext).map_err(|e| e.to_string())?;

    let mut rng = OsRng;
    let mut salt = [0u8; SALT_SIZE];
    rng.fill_bytes(&mut salt);
    
    let mut nonce_bytes = [0u8; NONCE_SIZE];
    rng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let key = derive_key(password, &salt)?;
    let cipher = Aes256Gcm::new(&key);
    
    let encrypted_data = cipher
        .encrypt(nonce, plaintext.as_ref())
        .map_err(|e| e.to_string())?;

    let mut output_file = File::create(output_path).map_err(|e| e.to_string())?;
    output_file.write_all(&encrypted_data).map_err(|e| e.to_string())?;

    Ok(EncryptionResult {
        encrypted_data,
        nonce: nonce_bytes,
        salt,
    })
}

pub fn decrypt_file(
    input_path: &Path,
    output_path: &Path,
    password: &str,
    nonce: &[u8; NONCE_SIZE],
    salt: &[u8; SALT_SIZE],
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

    let mut output_file = File::create(output_path).map_err(|e| e.to_string())?;
    output_file.write_all(&decrypted_data).map_err(|e| e.to_string())?;

    Ok(decrypted_data)
}

pub fn generate_random_salt() -> [u8; SALT_SIZE] {
    let mut rng = ArgonRng;
    let mut salt = [0u8; SALT_SIZE];
    rng.fill_bytes(&mut salt);
    salt
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
        
        let result = encrypt_file(input_file.path(), encrypted_file.path(), password).unwrap();
        
        let decrypted = decrypt_file(
            encrypted_file.path(),
            decrypted_file.path(),
            password,
            &result.nonce,
            &result.salt,
        ).unwrap();
        
        assert_eq!(plaintext.to_vec(), decrypted);
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
        
        let result = encrypt_file(input_file.path(), encrypted_file.path(), password).unwrap();
        
        let decryption_result = decrypt_file(
            encrypted_file.path(),
            decrypted_file.path(),
            wrong_password,
            &result.nonce,
            &result.salt,
        );
        
        assert!(decryption_result.is_err());
    }
}