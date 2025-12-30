
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: u8 = 0xAA;

pub fn encrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    let encryption_key = key.unwrap_or(DEFAULT_KEY);
    
    let mut input_file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    input_file.read_to_end(&mut buffer)?;
    
    let encrypted_data: Vec<u8> = buffer.iter()
        .map(|byte| byte ^ encryption_key)
        .collect();
    
    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&encrypted_data)?;
    
    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    encrypt_file(input_path, output_path, key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_encryption_decryption() {
        let original_text = b"Hello, World! This is a test message.";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), original_text).unwrap();
        
        encrypt_file(
            input_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap(),
            Some(0x55)
        ).unwrap();
        
        let encrypted_content = fs::read(encrypted_file.path()).unwrap();
        assert_ne!(encrypted_content, original_text);
        
        decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap(),
            Some(0x55)
        ).unwrap();
        
        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(decrypted_content, original_text);
    }
    
    #[test]
    fn test_default_key() {
        let test_data = b"Test data with default key";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), test_data).unwrap();
        
        encrypt_file(
            input_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap(),
            None
        ).unwrap();
        
        decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap(),
            None
        ).unwrap();
        
        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(decrypted_content, test_data);
    }
}
use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use hmac::{Hmac, Mac};
use pbkdf2::pbkdf2;
use rand::rngs::OsRng;
use rand::RngCore;
use sha2::Sha256;
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

type Aes256CbcEnc = cbc::Encryptor<aes::Aes256>;
type Aes256CbcDec = cbc::Decryptor<aes::Aes256>;

const SALT_LEN: usize = 16;
const IV_LEN: usize = 16;
const KEY_LEN: usize = 32;
const PBKDF2_ITERATIONS: u32 = 100_000;

#[derive(Debug)]
pub enum CryptoError {
    IoError(io::Error),
    InvalidData,
    KeyDerivationFailed,
}

impl From<io::Error> for CryptoError {
    fn from(err: io::Error) -> Self {
        CryptoError::IoError(err)
    }
}

pub struct FileCrypto;

impl FileCrypto {
    pub fn encrypt_file(
        input_path: &Path,
        output_path: &Path,
        password: &str,
    ) -> Result<(), CryptoError> {
        let mut input_file = fs::File::open(input_path)?;
        let mut plaintext = Vec::new();
        input_file.read_to_end(&mut plaintext)?;

        let mut salt = [0u8; SALT_LEN];
        let mut iv = [0u8; IV_LEN];
        OsRng.fill_bytes(&mut salt);
        OsRng.fill_bytes(&mut iv);

        let key = Self::derive_key(password, &salt)?;

        let cipher = Aes256CbcEnc::new(&key.into(), &iv.into());
        let ciphertext = cipher.encrypt_padded_vec_mut::<Pkcs7>(&plaintext);

        let mut output_file = fs::File::create(output_path)?;
        output_file.write_all(&salt)?;
        output_file.write_all(&iv)?;
        output_file.write_all(&ciphertext)?;

        Ok(())
    }

    pub fn decrypt_file(
        input_path: &Path,
        output_path: &Path,
        password: &str,
    ) -> Result<(), CryptoError> {
        let mut input_file = fs::File::open(input_path)?;
        let mut encrypted_data = Vec::new();
        input_file.read_to_end(&mut encrypted_data)?;

        if encrypted_data.len() < SALT_LEN + IV_LEN {
            return Err(CryptoError::InvalidData);
        }

        let salt = &encrypted_data[..SALT_LEN];
        let iv = &encrypted_data[SALT_LEN..SALT_LEN + IV_LEN];
        let ciphertext = &encrypted_data[SALT_LEN + IV_LEN..];

        let key = Self::derive_key(password, salt)?;

        let cipher = Aes256CbcDec::new(&key.into(), iv.into());
        let plaintext = cipher
            .decrypt_padded_vec_mut::<Pkcs7>(ciphertext)
            .map_err(|_| CryptoError::InvalidData)?;

        let mut output_file = fs::File::create(output_path)?;
        output_file.write_all(&plaintext)?;

        Ok(())
    }

    fn derive_key(password: &str, salt: &[u8]) -> Result<[u8; KEY_LEN], CryptoError> {
        let mut key = [0u8; KEY_LEN];
        pbkdf2::<Hmac<Sha256>>(password.as_bytes(), salt, PBKDF2_ITERATIONS, &mut key);
        Ok(key)
    }

    pub fn generate_hmac(data: &[u8], key: &[u8]) -> Result<Vec<u8>, CryptoError> {
        let mut mac = Hmac::<Sha256>::new_from_slice(key).map_err(|_| CryptoError::KeyDerivationFailed)?;
        mac.update(data);
        let result = mac.finalize().into_bytes().to_vec();
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encrypt_decrypt() {
        let plaintext = b"Secret data that needs protection";
        let password = "strong_password_123";

        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        fs::write(input_file.path(), plaintext).unwrap();

        FileCrypto::encrypt_file(input_file.path(), encrypted_file.path(), password).unwrap();
        FileCrypto::decrypt_file(encrypted_file.path(), decrypted_file.path(), password).unwrap();

        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(plaintext.to_vec(), decrypted_data);
    }

    #[test]
    fn test_hmac_generation() {
        let data = b"Data to authenticate";
        let key = b"secret_hmac_key";
        
        let hmac1 = FileCrypto::generate_hmac(data, key).unwrap();
        let hmac2 = FileCrypto::generate_hmac(data, key).unwrap();
        
        assert_eq!(hmac1, hmac2);
        assert_eq!(hmac1.len(), 32);
    }
}