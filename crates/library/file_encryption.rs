
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use std::fs;
use std::io::{self, Read, Write};

const NONCE_SIZE: usize = 12;

pub fn encrypt_file(input_path: &str, output_path: &str) -> io::Result<()> {
    let key = Aes256Gcm::generate_key(&mut OsRng);
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(&[0u8; NONCE_SIZE]);

    let mut file = fs::File::open(input_path)?;
    let mut plaintext = Vec::new();
    file.read_to_end(&mut plaintext)?;

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_ref())
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let mut output = fs::File::create(output_path)?;
    output.write_all(&key)?;
    output.write_all(nonce)?;
    output.write_all(&ciphertext)?;

    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str) -> io::Result<()> {
    let mut file = fs::File::open(input_path)?;
    let mut content = Vec::new();
    file.read_to_end(&mut content)?;

    if content.len() < 32 + NONCE_SIZE {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "File too short",
        ));
    }

    let (key_bytes, rest) = content.split_at(32);
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
        let original_content = b"Secret data that needs protection";
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        fs::write(input_file.path(), original_content).unwrap();

        encrypt_file(
            input_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap(),
        )
        .unwrap();

        decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap(),
        )
        .unwrap();

        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(original_content.to_vec(), decrypted_content);
    }
}
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: u8 = 0x55;

pub fn xor_encrypt_file(input_path: &Path, output_path: &Path, key: Option<u8>) -> io::Result<()> {
    let encryption_key = key.unwrap_or(DEFAULT_KEY);
    
    let mut input_file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    input_file.read_to_end(&mut buffer)?;
    
    let encrypted_data: Vec<u8> = buffer
        .iter()
        .map(|byte| byte ^ encryption_key)
        .collect();
    
    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&encrypted_data)?;
    
    Ok(())
}

pub fn xor_decrypt_file(input_path: &Path, output_path: &Path, key: Option<u8>) -> io::Result<()> {
    xor_encrypt_file(input_path, output_path, key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let original_content = b"Hello, World! This is a test file.";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), original_content).unwrap();
        
        xor_encrypt_file(input_file.path(), encrypted_file.path(), Some(0x42))
            .expect("Encryption failed");
        
        xor_decrypt_file(encrypted_file.path(), decrypted_file.path(), Some(0x42))
            .expect("Decryption failed");
        
        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(original_content, decrypted_content.as_slice());
    }
    
    #[test]
    fn test_default_key() {
        let original_content = b"Test with default key";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), original_content).unwrap();
        
        xor_encrypt_file(input_file.path(), encrypted_file.path(), None)
            .expect("Encryption with default key failed");
        
        xor_decrypt_file(encrypted_file.path(), decrypted_file.path(), None)
            .expect("Decryption with default key failed");
        
        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(original_content, decrypted_content.as_slice());
    }
}
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

pub fn xor_cipher(data: &mut [u8], key: &[u8]) {
    for (i, byte) in data.iter_mut().enumerate() {
        *byte ^= key[i % key.len()];
    }
}

pub fn encrypt_file(input_path: &Path, output_path: &Path, key: &str) -> io::Result<()> {
    let mut file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    
    xor_cipher(&mut buffer, key.as_bytes());
    
    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&buffer)?;
    
    Ok(())
}

pub fn decrypt_file(input_path: &Path, output_path: &Path, key: &str) -> io::Result<()> {
    encrypt_file(input_path, output_path, key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;

    #[test]
    fn test_xor_cipher_symmetry() {
        let key = "secret_key";
        let original_data = b"Hello, World!";
        let mut encrypted_data = original_data.to_vec();
        
        xor_cipher(&mut encrypted_data, key.as_bytes());
        assert_ne!(encrypted_data, original_data);
        
        xor_cipher(&mut encrypted_data, key.as_bytes());
        assert_eq!(encrypted_data, original_data);
    }

    #[test]
    fn test_file_encryption() -> io::Result<()> {
        let key = "test_key_123";
        let original_content = b"Confidential data that needs protection.";
        
        let input_file = NamedTempFile::new()?;
        let output_file = NamedTempFile::new()?;
        let decrypted_file = NamedTempFile::new()?;
        
        fs::write(input_file.path(), original_content)?;
        
        encrypt_file(input_file.path(), output_file.path(), key)?;
        let encrypted_content = fs::read(output_file.path())?;
        assert_ne!(encrypted_content, original_content);
        
        decrypt_file(output_file.path(), decrypted_file.path(), key)?;
        let decrypted_content = fs::read(decrypted_file.path())?;
        assert_eq!(decrypted_content, original_content);
        
        Ok(())
    }
}