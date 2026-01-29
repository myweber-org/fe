use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

/// XOR cipher implementation for file encryption/decryption
pub struct XorCipher {
    key: Vec<u8>,
}

impl XorCipher {
    /// Create new cipher with given key
    pub fn new(key: &[u8]) -> Self {
        XorCipher { key: key.to_vec() }
    }

    /// Process data using XOR cipher
    pub fn process(&self, data: &[u8]) -> Vec<u8> {
        data.iter()
            .enumerate()
            .map(|(i, &byte)| byte ^ self.key[i % self.key.len()])
            .collect()
    }
}

/// Encrypt file using XOR cipher
pub fn encrypt_file(input_path: &Path, output_path: &Path, key: &[u8]) -> io::Result<()> {
    let cipher = XorCipher::new(key);
    let mut input_file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    input_file.read_to_end(&mut buffer)?;

    let encrypted_data = cipher.process(&buffer);

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&encrypted_data)?;

    Ok(())
}

/// Decrypt file using XOR cipher
pub fn decrypt_file(input_path: &Path, output_path: &Path, key: &[u8]) -> io::Result<()> {
    encrypt_file(input_path, output_path, key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_xor_cipher_symmetry() {
        let key = b"secret_key";
        let cipher = XorCipher::new(key);
        let original_data = b"Hello, World! This is test data.";

        let encrypted = cipher.process(original_data);
        let decrypted = cipher.process(&encrypted);

        assert_eq!(original_data.to_vec(), decrypted);
    }

    #[test]
    fn test_file_encryption() -> io::Result<()> {
        let key = b"test_key_123";
        let original_content = b"Sample file content for encryption test.";

        let input_file = NamedTempFile::new()?;
        let output_file = NamedTempFile::new()?;

        fs::write(input_file.path(), original_content)?;

        encrypt_file(input_file.path(), output_file.path(), key)?;

        let encrypted_content = fs::read(output_file.path())?;
        assert_ne!(original_content, encrypted_content.as_slice());

        let decrypted_file = NamedTempFile::new()?;
        decrypt_file(output_file.path(), decrypted_file.path(), key)?;

        let decrypted_content = fs::read(decrypted_file.path())?;
        assert_eq!(original_content, decrypted_content.as_slice());

        Ok(())
    }
}use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use std::fs;
use std::io::{self, Write};
use std::path::Path;

const NONCE_SIZE: usize = 12;

pub fn encrypt_file(input_path: &Path, output_path: &Path) -> io::Result<()> {
    let key = Aes256Gcm::generate_key(&mut OsRng);
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(&[0u8; NONCE_SIZE]);

    let plaintext = fs::read(input_path)?;
    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_ref())
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let mut output = fs::File::create(output_path)?;
    output.write_all(&key)?;
    output.write_all(nonce)?;
    output.write_all(&ciphertext)?;

    Ok(())
}

pub fn decrypt_file(input_path: &Path, output_path: &Path) -> io::Result<()> {
    let data = fs::read(input_path)?;
    if data.len() < 32 + NONCE_SIZE {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "File too short to contain key and nonce",
        ));
    }

    let (key_bytes, rest) = data.split_at(32);
    let (nonce_bytes, ciphertext) = rest.split_at(NONCE_SIZE);

    let key = Key::<Aes256Gcm>::from_slice(key_bytes);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(nonce_bytes);

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    fs::write(output_path, plaintext)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_roundtrip() {
        let plaintext = b"Secret data for encryption test";
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        fs::write(input_file.path(), plaintext).unwrap();
        encrypt_file(input_file.path(), encrypted_file.path()).unwrap();
        decrypt_file(encrypted_file.path(), decrypted_file.path()).unwrap();

        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(plaintext.to_vec(), decrypted_data);
    }
}