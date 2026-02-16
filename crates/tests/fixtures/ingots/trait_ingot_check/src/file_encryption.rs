
use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use pbkdf2::pbkdf2_hmac;
use rand::RngCore;
use sha2::Sha256;
use std::fs;
use std::io::{Read, Write};
use std::path::Path;

type Aes256CbcEnc = cbc::Encryptor<aes::Aes256>;
type Aes256CbcDec = cbc::Decryptor<aes::Aes256>;

const SALT_LEN: usize = 16;
const IV_LEN: usize = 16;
const KEY_LEN: usize = 32;
const PBKDF2_ITERATIONS: u32 = 100_000;

pub struct FileCipher;

impl FileCipher {
    pub fn encrypt_file(
        source_path: &Path,
        dest_path: &Path,
        password: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut source_file = fs::File::open(source_path)?;
        let mut plaintext = Vec::new();
        source_file.read_to_end(&mut plaintext)?;

        let mut salt = [0u8; SALT_LEN];
        let mut iv = [0u8; IV_LEN];
        rand::thread_rng().fill_bytes(&mut salt);
        rand::thread_rng().fill_bytes(&mut iv);

        let key = Self::derive_key(password, &salt);

        let cipher = Aes256CbcEnc::new(&key.into(), &iv.into());
        let ciphertext = cipher.encrypt_padded_vec_mut::<Pkcs7>(&plaintext);

        let mut dest_file = fs::File::create(dest_path)?;
        dest_file.write_all(&salt)?;
        dest_file.write_all(&iv)?;
        dest_file.write_all(&ciphertext)?;

        Ok(())
    }

    pub fn decrypt_file(
        source_path: &Path,
        dest_path: &Path,
        password: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut source_file = fs::File::open(source_path)?;
        let mut encrypted_data = Vec::new();
        source_file.read_to_end(&mut encrypted_data)?;

        if encrypted_data.len() < SALT_LEN + IV_LEN {
            return Err("Invalid encrypted file format".into());
        }

        let salt = &encrypted_data[..SALT_LEN];
        let iv = &encrypted_data[SALT_LEN..SALT_LEN + IV_LEN];
        let ciphertext = &encrypted_data[SALT_LEN + IV_LEN..];

        let key = Self::derive_key(password, salt);

        let cipher = Aes256CbcDec::new(&key.into(), iv.into());
        let plaintext = cipher
            .decrypt_padded_vec_mut::<Pkcs7>(ciphertext)
            .map_err(|e| format!("Decryption failed: {}", e))?;

        let mut dest_file = fs::File::create(dest_path)?;
        dest_file.write_all(&plaintext)?;

        Ok(())
    }

    fn derive_key(password: &str, salt: &[u8]) -> [u8; KEY_LEN] {
        let mut key = [0u8; KEY_LEN];
        pbkdf2_hmac::<Sha256>(
            password.as_bytes(),
            salt,
            PBKDF2_ITERATIONS,
            &mut key,
        );
        key
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

        let source_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        fs::write(source_file.path(), plaintext).unwrap();

        FileCipher::encrypt_file(source_file.path(), encrypted_file.path(), password)
            .expect("Encryption failed");

        FileCipher::decrypt_file(encrypted_file.path(), decrypted_file.path(), password)
            .expect("Decryption failed");

        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(plaintext.to_vec(), decrypted_data);
    }

    #[test]
    fn test_wrong_password_fails() {
        let plaintext = b"Test data";
        let password = "correct_password";
        let wrong_password = "wrong_password";

        let source_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        fs::write(source_file.path(), plaintext).unwrap();

        FileCipher::encrypt_file(source_file.path(), encrypted_file.path(), password)
            .expect("Encryption failed");

        let result = FileCipher::decrypt_file(
            encrypted_file.path(),
            decrypted_file.path(),
            wrong_password,
        );
        assert!(result.is_err());
    }
}