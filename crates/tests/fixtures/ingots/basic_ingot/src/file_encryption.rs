
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: u8 = 0x55;

fn xor_crypt(data: &mut [u8], key: u8) {
    for byte in data.iter_mut() {
        *byte ^= key;
    }
}

fn process_file(input_path: &Path, output_path: &Path, key: u8) -> io::Result<()> {
    let mut file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    xor_crypt(&mut buffer, key);

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&buffer)?;

    Ok(())
}

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <input_file> <output_file>", args[0]);
        std::process::exit(1);
    }

    let input_path = Path::new(&args[1]);
    let output_path = Path::new(&args[2]);

    if !input_path.exists() {
        eprintln!("Error: Input file does not exist");
        std::process::exit(1);
    }

    process_file(input_path, output_path, DEFAULT_KEY)?;
    println!("File processed successfully with key 0x{:02X}", DEFAULT_KEY);

    Ok(())
}
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use chacha20poly1305::{ChaCha20Poly1305, Key as ChaChaKey, Nonce as ChaChaNonce};
use rand::RngCore;
use std::fs;
use std::io::{Read, Write};
use std::path::Path;

#[derive(Debug)]
pub enum EncryptionError {
    IoError(std::io::Error),
    CryptoError(String),
    InvalidInput(String),
}

impl From<std::io::Error> for EncryptionError {
    fn from(err: std::io::Error) -> Self {
        EncryptionError::IoError(err)
    }
}

#[derive(Clone, Copy, Debug)]
pub enum EncryptionAlgorithm {
    Aes256Gcm,
    ChaCha20Poly1305,
}

pub struct FileEncryptor {
    algorithm: EncryptionAlgorithm,
}

impl FileEncryptor {
    pub fn new(algorithm: EncryptionAlgorithm) -> Self {
        Self { algorithm }
    }

    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path, key: &[u8]) -> Result<(), EncryptionError> {
        let mut file_data = Vec::new();
        let mut file = fs::File::open(input_path)?;
        file.read_to_end(&mut file_data)?;

        let (ciphertext, nonce) = match self.algorithm {
            EncryptionAlgorithm::Aes256Gcm => self.encrypt_aes(&file_data, key)?,
            EncryptionAlgorithm::ChaCha20Poly1305 => self.encrypt_chacha(&file_data, key)?,
        };

        let mut output_file = fs::File::create(output_path)?;
        output_file.write_all(&nonce)?;
        output_file.write_all(&ciphertext)?;

        Ok(())
    }

    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path, key: &[u8]) -> Result<(), EncryptionError> {
        let mut encrypted_data = Vec::new();
        let mut file = fs::File::open(input_path)?;
        file.read_to_end(&mut encrypted_data)?;

        let nonce_size = match self.algorithm {
            EncryptionAlgorithm::Aes256Gcm => 12,
            EncryptionAlgorithm::ChaCha20Poly1305 => 12,
        };

        if encrypted_data.len() < nonce_size {
            return Err(EncryptionError::InvalidInput("File too small to contain nonce".to_string()));
        }

        let (nonce_bytes, ciphertext) = encrypted_data.split_at(nonce_size);
        let plaintext = match self.algorithm {
            EncryptionAlgorithm::Aes256Gcm => self.decrypt_aes(ciphertext, key, nonce_bytes)?,
            EncryptionAlgorithm::ChaCha20Poly1305 => self.decrypt_chacha(ciphertext, key, nonce_bytes)?,
        };

        let mut output_file = fs::File::create(output_path)?;
        output_file.write_all(&plaintext)?;

        Ok(())
    }

    fn encrypt_aes(&self, data: &[u8], key: &[u8]) -> Result<(Vec<u8>, Vec<u8>), EncryptionError> {
        if key.len() != 32 {
            return Err(EncryptionError::InvalidInput("AES-256-GCM requires 32-byte key".to_string()));
        }

        let key = Key::<Aes256Gcm>::from_slice(key);
        let cipher = Aes256Gcm::new(key);
        
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        cipher.encrypt(nonce, data)
            .map(|ciphertext| (ciphertext, nonce_bytes.to_vec()))
            .map_err(|e| EncryptionError::CryptoError(format!("Encryption failed: {:?}", e)))
    }

    fn decrypt_aes(&self, ciphertext: &[u8], key: &[u8], nonce_bytes: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        if key.len() != 32 {
            return Err(EncryptionError::InvalidInput("AES-256-GCM requires 32-byte key".to_string()));
        }
        if nonce_bytes.len() != 12 {
            return Err(EncryptionError::InvalidInput("AES-256-GCM requires 12-byte nonce".to_string()));
        }

        let key = Key::<Aes256Gcm>::from_slice(key);
        let cipher = Aes256Gcm::new(key);
        let nonce = Nonce::from_slice(nonce_bytes);

        cipher.decrypt(nonce, ciphertext)
            .map_err(|e| EncryptionError::CryptoError(format!("Decryption failed: {:?}", e)))
    }

    fn encrypt_chacha(&self, data: &[u8], key: &[u8]) -> Result<(Vec<u8>, Vec<u8>), EncryptionError> {
        if key.len() != 32 {
            return Err(EncryptionError::InvalidInput("ChaCha20Poly1305 requires 32-byte key".to_string()));
        }

        let key = ChaChaKey::from_slice(key);
        let cipher = ChaCha20Poly1305::new(key);
        
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = ChaChaNonce::from_slice(&nonce_bytes);

        cipher.encrypt(nonce, data)
            .map(|ciphertext| (ciphertext, nonce_bytes.to_vec()))
            .map_err(|e| EncryptionError::CryptoError(format!("Encryption failed: {:?}", e)))
    }

    fn decrypt_chacha(&self, ciphertext: &[u8], key: &[u8], nonce_bytes: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        if key.len() != 32 {
            return Err(EncryptionError::InvalidInput("ChaCha20Poly1305 requires 32-byte key".to_string()));
        }
        if nonce_bytes.len() != 12 {
            return Err(EncryptionError::InvalidInput("ChaCha20Poly1305 requires 12-byte nonce".to_string()));
        }

        let key = ChaChaKey::from_slice(key);
        let cipher = ChaCha20Poly1305::new(key);
        let nonce = ChaChaNonce::from_slice(nonce_bytes);

        cipher.decrypt(nonce, ciphertext)
            .map_err(|e| EncryptionError::CryptoError(format!("Decryption failed: {:?}", e)))
    }
}

pub fn generate_random_key() -> Vec<u8> {
    let mut key = [0u8; 32];
    OsRng.fill_bytes(&mut key);
    key.to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_aes_encryption_decryption() {
        let encryptor = FileEncryptor::new(EncryptionAlgorithm::Aes256Gcm);
        let key = generate_random_key();
        let test_data = b"Hello, this is a secret message!";

        let temp_input = NamedTempFile::new().unwrap();
        let temp_encrypted = NamedTempFile::new().unwrap();
        let temp_decrypted = NamedTempFile::new().unwrap();

        fs::write(temp_input.path(), test_data).unwrap();

        encryptor.encrypt_file(temp_input.path(), temp_encrypted.path(), &key).unwrap();
        encryptor.decrypt_file(temp_encrypted.path(), temp_decrypted.path(), &key).unwrap();

        let decrypted_data = fs::read(temp_decrypted.path()).unwrap();
        assert_eq!(test_data.to_vec(), decrypted_data);
    }

    #[test]
    fn test_chacha_encryption_decryption() {
        let encryptor = FileEncryptor::new(EncryptionAlgorithm::ChaCha20Poly1305);
        let key = generate_random_key();
        let test_data = b"Another secret message for testing";

        let temp_input = NamedTempFile::new().unwrap();
        let temp_encrypted = NamedTempFile::new().unwrap();
        let temp_decrypted = NamedTempFile::new().unwrap();

        fs::write(temp_input.path(), test_data).unwrap();

        encryptor.encrypt_file(temp_input.path(), temp_encrypted.path(), &key).unwrap();
        encryptor.decrypt_file(temp_encrypted.path(), temp_decrypted.path(), &key).unwrap();

        let decrypted_data = fs::read(temp_decrypted.path()).unwrap();
        assert_eq!(test_data.to_vec(), decrypted_data);
    }
}