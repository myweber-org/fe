
use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use pbkdf2::{pbkdf2_hmac, Params};
use rand::RngCore;
use sha2::Sha256;
use std::fs::{self, File};
use std::io::{Read, Write};

type Aes256CbcEnc = cbc::Encryptor<aes::Aes256>;
type Aes256CbcDec = cbc::Decryptor<aes::Aes256>;

const SALT_LEN: usize = 16;
const IV_LEN: usize = 16;
const KEY_ITERATIONS: u32 = 100_000;
const KEY_LEN: usize = 32;

pub struct CryptoConfig {
    pub salt: [u8; SALT_LEN],
    pub iv: [u8; IV_LEN],
}

impl CryptoConfig {
    pub fn new() -> Self {
        let mut salt = [0u8; SALT_LEN];
        let mut iv = [0u8; IV_LEN];
        rand::thread_rng().fill_bytes(&mut salt);
        rand::thread_rng().fill_bytes(&mut iv);
        Self { salt, iv }
    }

    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        if data.len() != SALT_LEN + IV_LEN {
            return None;
        }
        let mut salt = [0u8; SALT_LEN];
        let mut iv = [0u8; IV_LEN];
        salt.copy_from_slice(&data[..SALT_LEN]);
        iv.copy_from_slice(&data[SALT_LEN..]);
        Some(Self { salt, iv })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut result = Vec::with_capacity(SALT_LEN + IV_LEN);
        result.extend_from_slice(&self.salt);
        result.extend_from_slice(&self.iv);
        result
    }
}

fn derive_key(password: &str, salt: &[u8]) -> [u8; KEY_LEN] {
    let mut key = [0u8; KEY_LEN];
    let params = Params {
        rounds: KEY_ITERATIONS,
        output_length: KEY_LEN,
    };
    pbkdf2_hmac::<Sha256>(password.as_bytes(), salt, params.rounds, &mut key)
        .expect("PBKDF2 should not fail");
    key
}

pub fn encrypt_file(input_path: &str, output_path: &str, password: &str) -> Result<(), String> {
    let config = CryptoConfig::new();
    let key = derive_key(password, &config.salt);

    let mut input_file = File::open(input_path).map_err(|e| e.to_string())?;
    let mut plaintext = Vec::new();
    input_file.read_to_end(&mut plaintext).map_err(|e| e.to_string())?;

    let ciphertext = Aes256CbcEnc::new(&key.into(), &config.iv.into())
        .encrypt_padded_vec_mut::<Pkcs7>(&plaintext);

    let mut output_file = File::create(output_path).map_err(|e| e.to_string())?;
    output_file.write_all(&config.to_bytes()).map_err(|e| e.to_string())?;
    output_file.write_all(&ciphertext).map_err(|e| e.to_string())?;

    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str, password: &str) -> Result<(), String> {
    let mut input_file = File::open(input_path).map_err(|e| e.to_string())?;
    let mut file_data = Vec::new();
    input_file.read_to_end(&mut file_data).map_err(|e| e.to_string())?;

    if file_data.len() < SALT_LEN + IV_LEN {
        return Err("File too short to contain crypto config".to_string());
    }

    let config = CryptoConfig::from_bytes(&file_data[..SALT_LEN + IV_LEN])
        .ok_or("Invalid crypto config in file")?;
    let key = derive_key(password, &config.salt);

    let ciphertext = &file_data[SALT_LEN + IV_LEN..];
    let plaintext = Aes256CbcDec::new(&key.into(), &config.iv.into())
        .decrypt_padded_vec_mut::<Pkcs7>(ciphertext)
        .map_err(|e| e.to_string())?;

    let mut output_file = File::create(output_path).map_err(|e| e.to_string())?;
    output_file.write_all(&plaintext).map_err(|e| e.to_string())?;

    Ok(())
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

        encrypt_file(
            input_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap(),
            password,
        )
        .unwrap();

        decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap(),
            password,
        )
        .unwrap();

        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(plaintext.to_vec(), decrypted_data);
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

        encrypt_file(
            input_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap(),
            password,
        )
        .unwrap();

        let result = decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap(),
            wrong_password,
        );

        assert!(result.is_err());
    }
}