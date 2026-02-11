use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use chacha20poly1305::{ChaCha20Poly1305, Key as ChaChaKey, Nonce as ChaChaNonce};
use std::fs;
use std::io::{Read, Write};
use std::path::Path;

#[derive(Debug)]
pub enum EncryptionError {
    IoError(std::io::Error),
    CryptoError(String),
}

impl From<std::io::Error> for EncryptionError {
    fn from(err: std::io::Error) -> Self {
        EncryptionError::IoError(err)
    }
}

pub struct FileEncryptor {
    algorithm: EncryptionAlgorithm,
}

pub enum EncryptionAlgorithm {
    Aes256Gcm,
    ChaCha20Poly1305,
}

impl FileEncryptor {
    pub fn new(algorithm: EncryptionAlgorithm) -> Self {
        FileEncryptor { algorithm }
    }

    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path, key: &[u8]) -> Result<(), EncryptionError> {
        let mut file = fs::File::open(input_path)?;
        let mut plaintext = Vec::new();
        file.read_to_end(&mut plaintext)?;

        let ciphertext = match self.algorithm {
            EncryptionAlgorithm::Aes256Gcm => self.encrypt_aes(&plaintext, key)?,
            EncryptionAlgorithm::ChaCha20Poly1305 => self.encrypt_chacha(&plaintext, key)?,
        };

        let mut output_file = fs::File::create(output_path)?;
        output_file.write_all(&ciphertext)?;
        
        Ok(())
    }

    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path, key: &[u8]) -> Result<(), EncryptionError> {
        let mut file = fs::File::open(input_path)?;
        let mut ciphertext = Vec::new();
        file.read_to_end(&mut ciphertext)?;

        let plaintext = match self.algorithm {
            EncryptionAlgorithm::Aes256Gcm => self.decrypt_aes(&ciphertext, key)?,
            EncryptionAlgorithm::ChaCha20Poly1305 => self.decrypt_chacha(&ciphertext, key)?,
        };

        let mut output_file = fs::File::create(output_path)?;
        output_file.write_all(&plaintext)?;
        
        Ok(())
    }

    fn encrypt_aes(&self, plaintext: &[u8], key: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
        let nonce = Nonce::generate(&mut OsRng);
        
        cipher.encrypt(&nonce, plaintext)
            .map(|mut ciphertext| {
                let mut result = nonce.to_vec();
                result.append(&mut ciphertext);
                result
            })
            .map_err(|e| EncryptionError::CryptoError(format!("AES encryption failed: {}", e)))
    }

    fn decrypt_aes(&self, ciphertext: &[u8], key: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        if ciphertext.len() < 12 {
            return Err(EncryptionError::CryptoError("Invalid ciphertext length".to_string()));
        }
        
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
        let nonce = Nonce::from_slice(&ciphertext[..12]);
        let encrypted_data = &ciphertext[12..];
        
        cipher.decrypt(nonce, encrypted_data)
            .map_err(|e| EncryptionError::CryptoError(format!("AES decryption failed: {}", e)))
    }

    fn encrypt_chacha(&self, plaintext: &[u8], key: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        let cipher = ChaCha20Poly1305::new(ChaChaKey::from_slice(key));
        let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);
        
        cipher.encrypt(&nonce, plaintext)
            .map(|mut ciphertext| {
                let mut result = nonce.to_vec();
                result.append(&mut ciphertext);
                result
            })
            .map_err(|e| EncryptionError::CryptoError(format!("ChaCha20 encryption failed: {}", e)))
    }

    fn decrypt_chacha(&self, ciphertext: &[u8], key: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        if ciphertext.len() < 12 {
            return Err(EncryptionError::CryptoError("Invalid ciphertext length".to_string()));
        }
        
        let cipher = ChaCha20Poly1305::new(ChaChaKey::from_slice(key));
        let nonce = ChaChaNonce::from_slice(&ciphertext[..12]);
        let encrypted_data = &ciphertext[12..];
        
        cipher.decrypt(nonce, encrypted_data)
            .map_err(|e| EncryptionError::CryptoError(format!("ChaCha20 decryption failed: {}", e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_aes_encryption_decryption() {
        let encryptor = FileEncryptor::new(EncryptionAlgorithm::Aes256Gcm);
        let key = b"32-byte-key-for-aes-256-gcm-encryption";
        let test_data = b"Test encryption data for AES-256-GCM";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), test_data).unwrap();
        
        encryptor.encrypt_file(input_file.path(), encrypted_file.path(), key).unwrap();
        encryptor.decrypt_file(encrypted_file.path(), decrypted_file.path(), key).unwrap();
        
        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(test_data.to_vec(), decrypted_data);
    }

    #[test]
    fn test_chacha_encryption_decryption() {
        let encryptor = FileEncryptor::new(EncryptionAlgorithm::ChaCha20Poly1305);
        let key = b"32-byte-key-for-chacha20-poly1305";
        let test_data = b"Test encryption data for ChaCha20-Poly1305";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), test_data).unwrap();
        
        encryptor.encrypt_file(input_file.path(), encrypted_file.path(), key).unwrap();
        encryptor.decrypt_file(encrypted_file.path(), decrypted_file.path(), key).unwrap();
        
        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(test_data.to_vec(), decrypted_data);
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

    for byte in buffer.iter_mut() {
        *byte ^= encryption_key;
    }

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&buffer)?;
    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    encrypt_file(input_path, output_path, key)
}

pub fn process_directory(dir_path: &str, operation: &str, key: Option<u8>) -> io::Result<()> {
    let path = Path::new(dir_path);
    if !path.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Provided path is not a directory",
        ));
    }

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let file_path = entry.path();
        
        if file_path.is_file() {
            let input_str = file_path.to_str().unwrap();
            let output_str = format!("{}.processed", input_str);
            
            match operation {
                "encrypt" => encrypt_file(input_str, &output_str, key)?,
                "decrypt" => decrypt_file(input_str, &output_str, key)?,
                _ => return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Invalid operation. Use 'encrypt' or 'decrypt'",
                )),
            }
            
            println!("Processed: {}", input_str);
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let test_data = b"Hello, World!";
        let key = Some(0xAA);
        
        let mut input_file = NamedTempFile::new().unwrap();
        input_file.write_all(test_data).unwrap();
        
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        let input_path = input_file.path().to_str().unwrap();
        let encrypted_path = encrypted_file.path().to_str().unwrap();
        let decrypted_path = decrypted_file.path().to_str().unwrap();
        
        encrypt_file(input_path, encrypted_path, key).unwrap();
        decrypt_file(encrypted_path, decrypted_path, key).unwrap();
        
        let mut result = Vec::new();
        fs::File::open(decrypted_path)
            .unwrap()
            .read_to_end(&mut result)
            .unwrap();
            
        assert_eq!(test_data, result.as_slice());
    }
}use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

pub struct XORCipher {
    key: Vec<u8>,
}

impl XORCipher {
    pub fn new(key: &str) -> Self {
        XORCipher {
            key: key.as_bytes().to_vec(),
        }
    }

    pub fn encrypt_file(&self, source_path: &Path, dest_path: &Path) -> io::Result<()> {
        self.process_file(source_path, dest_path)
    }

    pub fn decrypt_file(&self, source_path: &Path, dest_path: &Path) -> io::Result<()> {
        self.process_file(source_path, dest_path)
    }

    fn process_file(&self, source_path: &Path, dest_path: &Path) -> io::Result<()> {
        let mut source_file = fs::File::open(source_path)?;
        let mut dest_file = fs::File::create(dest_path)?;

        let mut buffer = [0; 4096];
        let mut key_index = 0;

        loop {
            let bytes_read = source_file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }

            let mut processed_buffer = buffer[..bytes_read].to_vec();
            self.xor_transform(&mut processed_buffer, &mut key_index);

            dest_file.write_all(&processed_buffer)?;
        }

        dest_file.flush()?;
        Ok(())
    }

    fn xor_transform(&self, data: &mut [u8], key_index: &mut usize) {
        for byte in data.iter_mut() {
            *byte ^= self.key[*key_index];
            *key_index = (*key_index + 1) % self.key.len();
        }
    }
}

pub fn validate_key(key: &str) -> Result<(), String> {
    if key.is_empty() {
        return Err("Encryption key cannot be empty".to_string());
    }
    if key.len() < 8 {
        return Err("Encryption key should be at least 8 characters".to_string());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_xor_cipher_symmetry() {
        let cipher = XORCipher::new("strong_encryption_key_123!");
        let test_data = b"Hello, this is a secret message!";

        let mut encrypted = test_data.to_vec();
        let mut key_index = 0;
        cipher.xor_transform(&mut encrypted, &mut key_index);

        let mut decrypted = encrypted.clone();
        key_index = 0;
        cipher.xor_transform(&mut decrypted, &mut key_index);

        assert_eq!(test_data, decrypted.as_slice());
    }

    #[test]
    fn test_file_encryption() {
        let cipher = XORCipher::new("test_key_5678");
        let source_content = b"Sample file content for encryption test";

        let source_file = NamedTempFile::new().unwrap();
        let dest_file = NamedTempFile::new().unwrap();

        fs::write(source_file.path(), source_content).unwrap();

        cipher.encrypt_file(source_file.path(), dest_file.path()).unwrap();
        
        let encrypted_content = fs::read(dest_file.path()).unwrap();
        assert_ne!(source_content, encrypted_content.as_slice());

        let decrypted_file = NamedTempFile::new().unwrap();
        cipher.decrypt_file(dest_file.path(), decrypted_file.path()).unwrap();
        
        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(source_content, decrypted_content.as_slice());
    }

    #[test]
    fn test_key_validation() {
        assert!(validate_key("").is_err());
        assert!(validate_key("short").is_err());
        assert!(validate_key("valid_key_123").is_ok());
    }
}use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use argon2::{
    password_hash::{rand_core::OsRng as ArgonRng, PasswordHasher, SaltString},
    Argon2, Params,
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
    let salt_str = SaltString::encode_b64(salt).map_err(|e| e.to_string())?;
    let argon2 = Argon2::new(
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        Params::new(15000, 2, 1, Some(32)).map_err(|e| e.to_string())?,
    );
    
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt_str)
        .map_err(|e| e.to_string())?;
    
    let key_bytes = password_hash.hash.ok_or("Hash generation failed")?;
    Ok(*Key::<Aes256Gcm>::from_slice(key_bytes.as_bytes()))
}

pub fn encrypt_file(
    input_path: &Path,
    output_path: &Path,
    password: &str,
) -> Result<EncryptionResult, String> {
    let mut file = fs::File::open(input_path).map_err(|e| e.to_string())?;
    let mut plaintext = Vec::new();
    file.read_to_end(&mut plaintext).map_err(|e| e.to_string())?;

    let mut salt = [0u8; SALT_SIZE];
    ArgonRng.fill_bytes(&mut salt);
    
    let key = derive_key(password, &salt)?;
    let cipher = Aes256Gcm::new(&key);
    
    let mut nonce_bytes = [0u8; NONCE_SIZE];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    
    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_ref())
        .map_err(|e| e.to_string())?;
    
    let mut output_file = fs::File::create(output_path).map_err(|e| e.to_string())?;
    output_file.write_all(&ciphertext).map_err(|e| e.to_string())?;
    
    Ok(EncryptionResult {
        ciphertext,
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
    let mut file = fs::File::open(input_path).map_err(|e| e.to_string())?;
    let mut ciphertext = Vec::new();
    file.read_to_end(&mut ciphertext).map_err(|e| e.to_string())?;
    
    let key = derive_key(password, salt)?;
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(nonce);
    
    let plaintext = cipher
        .decrypt(nonce, ciphertext.as_ref())
        .map_err(|e| e.to_string())?;
    
    let mut output_file = fs::File::create(output_path).map_err(|e| e.to_string())?;
    output_file.write_all(&plaintext).map_err(|e| e.to_string())?;
    
    Ok(plaintext)
}

pub fn generate_secure_password(length: usize) -> Result<String, String> {
    if length < 12 {
        return Err("Password length must be at least 12 characters".to_string());
    }
    
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789\
                            !@#$%^&*()_+-=[]{}|;:,.<>?";
    
    let mut rng = OsRng;
    let mut password = Vec::with_capacity(length);
    
    for _ in 0..length {
        let idx = rng.next_u32() as usize % CHARSET.len();
        password.push(CHARSET[idx]);
    }
    
    Ok(String::from_utf8(password).map_err(|e| e.to_string())?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_encryption_decryption() {
        let plaintext = b"Test encryption and decryption";
        let password = "SecurePassword123!";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), plaintext).unwrap();
        
        let enc_result = encrypt_file(
            input_file.path(),
            encrypted_file.path(),
            password,
        ).unwrap();
        
        let decrypted = decrypt_file(
            encrypted_file.path(),
            decrypted_file.path(),
            password,
            &enc_result.nonce,
            &enc_result.salt,
        ).unwrap();
        
        assert_eq!(plaintext.to_vec(), decrypted);
    }
    
    #[test]
    fn test_password_generation() {
        let password = generate_secure_password(16).unwrap();
        assert_eq!(password.len(), 16);
        assert!(password.chars().any(|c| c.is_ascii_uppercase()));
        assert!(password.chars().any(|c| c.is_ascii_lowercase()));
        assert!(password.chars().any(|c| c.is_ascii_digit()));
        assert!(password.chars().any(|c| !c.is_alphanumeric()));
    }
}use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

pub struct XORCipher {
    key: Vec<u8>,
}

impl XORCipher {
    pub fn new(key: &str) -> Self {
        XORCipher {
            key: key.as_bytes().to_vec(),
        }
    }

    pub fn encrypt_file(&self, source_path: &Path, dest_path: &Path) -> io::Result<()> {
        self.process_file(source_path, dest_path)
    }

    pub fn decrypt_file(&self, source_path: &Path, dest_path: &Path) -> io::Result<()> {
        self.process_file(source_path, dest_path)
    }

    fn process_file(&self, source_path: &Path, dest_path: &Path) -> io::Result<()> {
        let mut source_file = fs::File::open(source_path)?;
        let mut dest_file = fs::File::create(dest_path)?;

        let mut buffer = [0; 4096];
        let mut key_index = 0;

        loop {
            let bytes_read = source_file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }

            let mut processed_buffer = buffer[..bytes_read].to_vec();
            self.xor_transform(&mut processed_buffer, &mut key_index);

            dest_file.write_all(&processed_buffer)?;
        }

        dest_file.flush()?;
        Ok(())
    }

    fn xor_transform(&self, data: &mut [u8], key_index: &mut usize) {
        for byte in data.iter_mut() {
            *byte ^= self.key[*key_index];
            *key_index = (*key_index + 1) % self.key.len();
        }
    }
}

pub fn validate_key(key: &str) -> Result<(), &'static str> {
    if key.is_empty() {
        return Err("Encryption key cannot be empty");
    }
    if key.len() < 8 {
        return Err("Encryption key must be at least 8 characters");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_xor_encryption_decryption() {
        let cipher = XORCipher::new("secure_key_123");
        let test_data = b"Hello, this is a test message for XOR encryption!";

        let mut encrypted = test_data.to_vec();
        let mut key_idx = 0;
        cipher.xor_transform(&mut encrypted, &mut key_idx);

        let mut decrypted = encrypted.clone();
        key_idx = 0;
        cipher.xor_transform(&mut decrypted, &mut key_idx);

        assert_eq!(test_data, decrypted.as_slice());
        assert_ne!(test_data, encrypted.as_slice());
    }

    #[test]
    fn test_file_encryption() {
        let cipher = XORCipher::new("test_encryption_key");
        let source_file = NamedTempFile::new().unwrap();
        let dest_file = NamedTempFile::new().unwrap();

        fs::write(source_file.path(), "Sample file content").unwrap();

        cipher.encrypt_file(source_file.path(), dest_file.path()).unwrap();
        
        let encrypted_content = fs::read(dest_file.path()).unwrap();
        assert_ne!(encrypted_content, b"Sample file content");
    }

    #[test]
    fn test_key_validation() {
        assert!(validate_key("").is_err());
        assert!(validate_key("short").is_err());
        assert!(validate_key("valid_long_key_123").is_ok());
    }
}