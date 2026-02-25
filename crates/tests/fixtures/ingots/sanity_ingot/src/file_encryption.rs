
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: u8 = 0x55;

fn xor_cipher(data: &mut [u8], key: u8) {
    for byte in data.iter_mut() {
        *byte ^= key;
    }
}

fn process_file(input_path: &Path, output_path: &Path, key: u8) -> io::Result<()> {
    let mut file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    
    xor_cipher(&mut buffer, key);
    
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
    
    println!("File processed successfully with XOR key 0x{:02X}", DEFAULT_KEY);
    Ok(())
}
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: u8 = 0xAA;

fn xor_cipher(data: &mut [u8], key: u8) {
    for byte in data.iter_mut() {
        *byte ^= key;
    }
}

fn process_file(input_path: &Path, output_path: &Path, key: u8) -> io::Result<()> {
    let mut file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    xor_cipher(&mut buffer, key);

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&buffer)?;

    Ok(())
}

pub fn encrypt_file(input_path: &str, output_path: &str) -> io::Result<()> {
    let key = DEFAULT_KEY;
    process_file(Path::new(input_path), Path::new(output_path), key)
}

pub fn decrypt_file(input_path: &str, output_path: &str) -> io::Result<()> {
    let key = DEFAULT_KEY;
    process_file(Path::new(input_path), Path::new(output_path), key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;

    #[test]
    fn test_xor_cipher() {
        let mut data = vec![0x00, 0xFF, 0x55, 0xAA];
        let key = 0xAA;
        xor_cipher(&mut data, key);
        assert_eq!(data, vec![0xAA, 0x55, 0xFF, 0x00]);
        xor_cipher(&mut data, key);
        assert_eq!(data, vec![0x00, 0xFF, 0x55, 0xAA]);
    }

    #[test]
    fn test_file_encryption() -> io::Result<()> {
        let original_content = b"Hello, World! This is a test file.";
        let input_file = NamedTempFile::new()?;
        let encrypted_file = NamedTempFile::new()?;
        let decrypted_file = NamedTempFile::new()?;

        fs::write(input_file.path(), original_content)?;

        encrypt_file(
            input_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap(),
        )?;

        decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap(),
        )?;

        let decrypted_content = fs::read(decrypted_file.path())?;
        assert_eq!(original_content.as_slice(), decrypted_content.as_slice());

        Ok(())
    }
}
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use chacha20poly1305::{ChaCha20Poly1305, Key as ChaChaKey};
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

    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), EncryptionError> {
        let mut file = fs::File::open(input_path)?;
        let mut plaintext = Vec::new();
        file.read_to_end(&mut plaintext)?;

        let (ciphertext, nonce) = match self.algorithm {
            EncryptionAlgorithm::Aes256Gcm => {
                let key = Aes256Gcm::generate_key(&mut OsRng);
                let cipher = Aes256Gcm::new(&key);
                let nonce = Nonce::generate(&mut OsRng);
                let ciphertext = cipher.encrypt(&nonce, plaintext.as_ref())
                    .map_err(|e| EncryptionError::CryptoError(e.to_string()))?;
                (ciphertext, nonce.to_vec())
            }
            EncryptionAlgorithm::ChaCha20Poly1305 => {
                let key = ChaCha20Poly1305::generate_key(&mut OsRng);
                let cipher = ChaCha20Poly1305::new(&key);
                let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);
                let ciphertext = cipher.encrypt(&nonce, plaintext.as_ref())
                    .map_err(|e| EncryptionError::CryptoError(e.to_string()))?;
                (ciphertext, nonce.to_vec())
            }
        };

        let mut output_file = fs::File::create(output_path)?;
        output_file.write_all(&nonce)?;
        output_file.write_all(&ciphertext)?;

        Ok(())
    }

    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path, key: &[u8]) -> Result<(), EncryptionError> {
        let mut file = fs::File::open(input_path)?;
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;

        let nonce_size = match self.algorithm {
            EncryptionAlgorithm::Aes256Gcm => 12,
            EncryptionAlgorithm::ChaCha20Poly1305 => 12,
        };

        if data.len() < nonce_size {
            return Err(EncryptionError::CryptoError("File too short".to_string()));
        }

        let (nonce, ciphertext) = data.split_at(nonce_size);

        let plaintext = match self.algorithm {
            EncryptionAlgorithm::Aes256Gcm => {
                let cipher = Aes256Gcm::new_from_slice(key)
                    .map_err(|e| EncryptionError::CryptoError(e.to_string()))?;
                cipher.decrypt(Nonce::from_slice(nonce), ciphertext)
                    .map_err(|e| EncryptionError::CryptoError(e.to_string()))?
            }
            EncryptionAlgorithm::ChaCha20Poly1305 => {
                let cipher = ChaCha20Poly1305::new_from_slice(key)
                    .map_err(|e| EncryptionError::CryptoError(e.to_string()))?;
                cipher.decrypt(nonce.into(), ciphertext)
                    .map_err(|e| EncryptionError::CryptoError(e.to_string()))?
            }
        };

        let mut output_file = fs::File::create(output_path)?;
        output_file.write_all(&plaintext)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_aes_encryption_decryption() {
        let encryptor = FileEncryptor::new(EncryptionAlgorithm::Aes256Gcm);
        let test_data = b"Hello, this is a secret message!";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        fs::write(input_file.path(), test_data).unwrap();
        
        encryptor.encrypt_file(input_file.path(), encrypted_file.path()).unwrap();
        
        let key = [0u8; 32];
        encryptor.decrypt_file(encrypted_file.path(), decrypted_file.path(), &key).unwrap();
        
        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(decrypted_data, test_data);
    }

    #[test]
    fn test_chacha_encryption_decryption() {
        let encryptor = FileEncryptor::new(EncryptionAlgorithm::ChaCha20Poly1305);
        let test_data = b"Another secret message for testing";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        fs::write(input_file.path(), test_data).unwrap();
        
        encryptor.encrypt_file(input_file.path(), encrypted_file.path()).unwrap();
        
        let key = [0u8; 32];
        encryptor.decrypt_file(encrypted_file.path(), decrypted_file.path(), &key).unwrap();
        
        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(decrypted_data, test_data);
    }
}
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const NONCE_SIZE: usize = 12;

pub fn encrypt_file(input_path: &Path, output_path: &Path, key: &[u8; 32]) -> io::Result<()> {
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let mut nonce_bytes = [0u8; NONCE_SIZE];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let mut input_file = fs::File::open(input_path)?;
    let mut plaintext = Vec::new();
    input_file.read_to_end(&mut plaintext)?;

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_ref())
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&nonce_bytes)?;
    output_file.write_all(&ciphertext)?;

    Ok(())
}

pub fn decrypt_file(input_path: &Path, output_path: &Path, key: &[u8; 32]) -> io::Result<()> {
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));

    let mut input_file = fs::File::open(input_path)?;
    let mut encrypted_data = Vec::new();
    input_file.read_to_end(&mut encrypted_data)?;

    if encrypted_data.len() < NONCE_SIZE {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "File too short to contain nonce",
        ));
    }

    let (nonce_bytes, ciphertext) = encrypted_data.split_at(NONCE_SIZE);
    let nonce = Nonce::from_slice(nonce_bytes);

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&plaintext)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let key = [0x42u8; 32];
        let original_content = b"Secret data that needs protection";

        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        fs::write(input_file.path(), original_content).unwrap();

        encrypt_file(input_file.path(), encrypted_file.path(), &key).unwrap();
        decrypt_file(encrypted_file.path(), decrypted_file.path(), &key).unwrap();

        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(original_content.to_vec(), decrypted_content);
    }
}
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use std::fs;
use std::io::{self, Read, Write};

const NONCE_SIZE: usize = 12;

pub fn encrypt_file(input_path: &str, output_path: &str) -> io::Result<()> {
    let mut file = fs::File::open(input_path)?;
    let mut plaintext = Vec::new();
    file.read_to_end(&mut plaintext)?;

    let key = Aes256Gcm::generate_key(&mut OsRng);
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(&[0u8; NONCE_SIZE]);

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_ref())
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&key)?;
    output_file.write_all(&ciphertext)?;

    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str) -> io::Result<()> {
    let mut file = fs::File::open(input_path)?;
    let mut content = Vec::new();
    file.read_to_end(&mut content)?;

    if content.len() < 32 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "File too short",
        ));
    }

    let (key_bytes, ciphertext) = content.split_at(32);
    let key = Key::<Aes256Gcm>::from_slice(key_bytes);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(&[0u8; NONCE_SIZE]);

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&plaintext)?;

    Ok(())
}