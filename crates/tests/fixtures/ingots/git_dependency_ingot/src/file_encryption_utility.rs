
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

    pub fn encrypt_file(
        &self,
        input_path: &Path,
        output_path: &Path,
        key: &[u8],
    ) -> Result<(), EncryptionError> {
        let mut file_data = Vec::new();
        fs::File::open(input_path)?.read_to_end(&mut file_data)?;

        let encrypted_data = match self.algorithm {
            EncryptionAlgorithm::Aes256Gcm => self.aes_encrypt(&file_data, key)?,
            EncryptionAlgorithm::ChaCha20Poly1305 => self.chacha_encrypt(&file_data, key)?,
        };

        let mut output_file = fs::File::create(output_path)?;
        output_file.write_all(&encrypted_data)?;
        Ok(())
    }

    pub fn decrypt_file(
        &self,
        input_path: &Path,
        output_path: &Path,
        key: &[u8],
    ) -> Result<(), EncryptionError> {
        let mut encrypted_data = Vec::new();
        fs::File::open(input_path)?.read_to_end(&mut encrypted_data)?;

        let decrypted_data = match self.algorithm {
            EncryptionAlgorithm::Aes256Gcm => self.aes_decrypt(&encrypted_data, key)?,
            EncryptionAlgorithm::ChaCha20Poly1305 => self.chacha_decrypt(&encrypted_data, key)?,
        };

        let mut output_file = fs::File::create(output_path)?;
        output_file.write_all(&decrypted_data)?;
        Ok(())
    }

    fn aes_encrypt(&self, data: &[u8], key: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
        let nonce = Nonce::generate(&mut OsRng);
        
        cipher
            .encrypt(&nonce, data)
            .map(|mut ciphertext| {
                let mut result = nonce.to_vec();
                result.append(&mut ciphertext);
                result
            })
            .map_err(|e| EncryptionError::CryptoError(e.to_string()))
    }

    fn aes_decrypt(&self, data: &[u8], key: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        if data.len() < 12 {
            return Err(EncryptionError::CryptoError("Invalid encrypted data".into()));
        }

        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
        let (nonce_bytes, ciphertext) = data.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);

        cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| EncryptionError::CryptoError(e.to_string()))
    }

    fn chacha_encrypt(&self, data: &[u8], key: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        let cipher = ChaCha20Poly1305::new(ChaChaKey::from_slice(key));
        let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);
        
        cipher
            .encrypt(&nonce, data)
            .map(|mut ciphertext| {
                let mut result = nonce.to_vec();
                result.append(&mut ciphertext);
                result
            })
            .map_err(|e| EncryptionError::CryptoError(e.to_string()))
    }

    fn chacha_decrypt(&self, data: &[u8], key: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        if data.len() < 12 {
            return Err(EncryptionError::CryptoError("Invalid encrypted data".into()));
        }

        let cipher = ChaCha20Poly1305::new(ChaChaKey::from_slice(key));
        let (nonce_bytes, ciphertext) = data.split_at(12);
        let nonce = ChaChaNonce::from_slice(nonce_bytes);

        cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| EncryptionError::CryptoError(e.to_string()))
    }
}

pub fn generate_random_key() -> Vec<u8> {
    let mut key = vec![0u8; 32];
    OsRng.fill_bytes(&mut key);
    key
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_aes_encryption_decryption() {
        let test_data = b"Test encryption data";
        let key = generate_random_key();
        
        let encryptor = FileEncryptor::new(EncryptionAlgorithm::Aes256Gcm);
        
        let encrypted = encryptor.aes_encrypt(test_data, &key).unwrap();
        let decrypted = encryptor.aes_decrypt(&encrypted, &key).unwrap();
        
        assert_eq!(test_data.to_vec(), decrypted);
    }

    #[test]
    fn test_chacha_encryption_decryption() {
        let test_data = b"Test chacha encryption";
        let key = generate_random_key();
        
        let encryptor = FileEncryptor::new(EncryptionAlgorithm::ChaCha20Poly1305);
        
        let encrypted = encryptor.chacha_encrypt(test_data, &key).unwrap();
        let decrypted = encryptor.chacha_decrypt(&encrypted, &key).unwrap();
        
        assert_eq!(test_data.to_vec(), decrypted);
    }

    #[test]
    fn test_file_encryption() {
        let input_file = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        let test_data = b"File encryption test content";
        fs::write(input_file.path(), test_data).unwrap();
        
        let key = generate_random_key();
        let encryptor = FileEncryptor::new(EncryptionAlgorithm::Aes256Gcm);
        
        encryptor.encrypt_file(input_file.path(), output_file.path(), &key).unwrap();
        encryptor.decrypt_file(output_file.path(), decrypted_file.path(), &key).unwrap();
        
        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(test_data.to_vec(), decrypted_content);
    }
}use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: u8 = 0xAA;

pub fn xor_encrypt(data: &[u8], key: u8) -> Vec<u8> {
    data.iter().map(|byte| byte ^ key).collect()
}

pub fn xor_decrypt(data: &[u8], key: u8) -> Vec<u8> {
    xor_encrypt(data, key)
}

pub fn process_file(input_path: &Path, output_path: &Path, key: u8) -> io::Result<()> {
    let mut input_file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    input_file.read_to_end(&mut buffer)?;
    
    let processed_data = xor_encrypt(&buffer, key);
    
    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&processed_data)?;
    
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
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_xor_symmetry() {
        let original = b"Hello, World!";
        let key = 0x55;
        
        let encrypted = xor_encrypt(original, key);
        assert_ne!(original, encrypted.as_slice());
        
        let decrypted = xor_decrypt(&encrypted, key);
        assert_eq!(original, decrypted.as_slice());
    }

    #[test]
    fn test_file_encryption() -> io::Result<()> {
        let mut temp_input = NamedTempFile::new()?;
        let temp_output = NamedTempFile::new()?;
        
        let test_data = b"Test encryption data";
        temp_input.write_all(test_data)?;
        
        encrypt_file(
            temp_input.path().to_str().unwrap(),
            temp_output.path().to_str().unwrap(),
        )?;
        
        let mut encrypted_content = Vec::new();
        fs::File::open(temp_output.path())?.read_to_end(&mut encrypted_content)?;
        
        assert_ne!(test_data, encrypted_content.as_slice());
        
        let mut temp_decrypted = NamedTempFile::new()?;
        decrypt_file(
            temp_output.path().to_str().unwrap(),
            temp_decrypted.path().to_str().unwrap(),
        )?;
        
        let mut decrypted_content = Vec::new();
        fs::File::open(temp_decrypted.path())?.read_to_end(&mut decrypted_content)?;
        
        assert_eq!(test_data, decrypted_content.as_slice());
        
        Ok(())
    }
}